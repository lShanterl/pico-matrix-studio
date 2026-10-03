use core::sync::atomic::{AtomicBool, Ordering};
use crc::{Crc, CRC_32_ISO_HDLC};
use embassy_rp::flash::{Async, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use embassy_time::{with_timeout, Duration};
use embedded_storage_async::nor_flash::NorFlash;
use log::info;
use matrix_protocol::MAX_ANIMATION_FRAMES;
use crate::animations::StoredAnimation;

pub const SETTINGS_ADDR: u32 = 0x100000;
pub const FLASH_SIZE: usize = 2 * 1024 * 1024;
pub const ERASE_SIZE: usize = 4096;
pub const PAGE_SIZE: usize = 256;
// a safety measure to avoid memory corruption - incremented by one whenever data inside device storage changes
const SETTINGS_VERSION: u32 = 1;
const ANIMATION_VERSION: u32 = 1;
const HEADER: usize = PAGE_SIZE; // animation header takes one page
const SETTINGS_LEN: usize = 16;
pub const MIN_MA: u16 = 300;
pub const MAX_MA: u16 = 20000;
pub const ANIMATION_ADDR: u32 = SETTINGS_ADDR + ERASE_SIZE as u32;

//0x100000  settings   1 sector   16 bytes, own version + CRC
//0x104096  animation  6 sectors  [256-byte header: version + CRC][StoredAnimation bytes]
// control sum to avoid data corruption when device is unplugged during save / load
const CRC_CALCULATOR: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

#[repr(u8)]
#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    Procedural = 0,
    Uploaded = 1,
}

impl Mode {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => Mode::Uploaded,
            _ => Mode::Procedural,
        }
    }
}

#[repr(C)]
#[derive(Clone)]
pub struct DeviceStorage {
    pub brightness: f32,
    pub max_amper: u16,
    pub current_animation: u8,
    pub mode: u8,
    pub custom_animation: StoredAnimation,
}

impl DeviceStorage {
    pub fn mode(&self) -> Mode { Mode::from_u8(self.mode) }
    pub fn set_mode(&mut self, m: Mode) { self.mode = m as u8; }
    pub fn set_max_amper(&mut self, ma: u16) { self.max_amper = ma.clamp(MIN_MA, MAX_MA); }

    fn settings_bytes(&self) -> [u8; 16]{
        let mut b = [0u8; 16];
        b[0..4].copy_from_slice(&SETTINGS_VERSION.to_le_bytes());
        b[4..8].copy_from_slice(&self.brightness.to_le_bytes());
        b[8..10].copy_from_slice(&self.max_amper.to_le_bytes());
        b[10] = self.mode;
        b[11] = self.current_animation;
        let crc = CRC_CALCULATOR.checksum(&b[..12]);
        b[12..16].copy_from_slice(&crc.to_le_bytes());
        b
    }

    fn apply_settings_bytes(&mut self, b: &[u8; 16]) -> bool {
        // rp2040 is natively little-endian, tcp protocol always big-endian (data in motion rule)
        if u32::from_le_bytes(b[0..4].try_into().unwrap()) != SETTINGS_VERSION { return false; }
        if u32::from_le_bytes(b[12..16].try_into().unwrap()) != CRC_CALCULATOR.checksum(&b[..12]) { return false; }
        let br = f32::from_le_bytes(b[4..8].try_into().unwrap());
        if !br.is_finite() { return false; }
        self.brightness = br.clamp(0.0, 1.0);
        self.set_max_amper(u16::from_le_bytes(b[8..10].try_into().unwrap()));
        self.mode = b[10];
        self.current_animation = b[11];
        true
    }
    fn animation_bytes(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(
            &self.custom_animation as *const _ as *const u8,
            core::mem::size_of::<StoredAnimation>()) }
    }
    fn animation_crc(&self) -> u32 { CRC_CALCULATOR.checksum(self.animation_bytes()) }

    pub async fn load_from_flash(&mut self, flash: &mut Flash<'_, FLASH,embassy_rp::flash::Async, { FLASH_SIZE } >) {
        // settings
        let mut sb = [0u8; 16];
        let ok = flash.read(SETTINGS_ADDR, &mut sb).await.is_ok() && self.apply_settings_bytes(&sb);
        if !ok {
            let d = Self::new();
            self.brightness = d.brightness;
            self.max_amper = d.max_amper;
            self.mode = d.mode;
            self.current_animation = d.current_animation;
        }

        // animation
        let mut hdr = [0u8; 8];
        let mut ok = flash.read(ANIMATION_ADDR, &mut hdr).await.is_ok();
        if ok {
            let size = core::mem::size_of::<StoredAnimation>();
            let dst = unsafe { core::slice::from_raw_parts_mut(
                &mut self.custom_animation as *mut _ as *mut u8, size) };
            ok = flash.read(ANIMATION_ADDR + HEADER as u32, dst).await.is_ok()
                && u32::from_le_bytes(hdr[0..4].try_into().unwrap()) == ANIMATION_VERSION
                && self.custom_animation.frame_count <= MAX_ANIMATION_FRAMES
                && u32::from_le_bytes(hdr[4..8].try_into().unwrap()) == self.animation_crc();
        }
        if !ok { self.custom_animation = StoredAnimation::new(); }
    }

    pub const fn new() -> Self {

        Self {
            brightness: 1.0f32,    // default to max brightness
            max_amper: 500,        // default to 500mA
            current_animation: 0,
            mode: 0,
            custom_animation: StoredAnimation::new(),
        }
    }
}

async fn save_animation(s: &DeviceStorage, flash: &mut Flash<'_, FLASH,embassy_rp::flash::Async, { FLASH_SIZE } >) -> Result<(), embassy_rp::flash::Error> {
    let data = s.animation_bytes();
    let total = HEADER + data.len();
    let erase_end = ANIMATION_ADDR + (((total + ERASE_SIZE - 1) / ERASE_SIZE) * ERASE_SIZE) as u32;
    flash.erase(ANIMATION_ADDR, erase_end).await?;

    let data_addr = ANIMATION_ADDR + HEADER as u32;
    let full = data.len() - data.len() % PAGE_SIZE;
    flash.write(data_addr, &data[..full]).await?;
    if full < data.len() {
        let mut last = [0xFFu8; PAGE_SIZE];
        last[..data.len() - full].copy_from_slice(&data[full..]);
        flash.write(data_addr + full as u32, &last).await?;
    }

    // header LAST = commit marker
    let mut hdr = [0xFFu8; PAGE_SIZE];
    hdr[0..4].copy_from_slice(&ANIMATION_VERSION.to_le_bytes());
    hdr[4..8].copy_from_slice(&s.animation_crc().to_le_bytes());
    flash.write(ANIMATION_ADDR, &hdr).await
}
async fn save_settings(bytes: [u8; 16], flash: &mut Flash<'_, FLASH,embassy_rp::flash::Async, { FLASH_SIZE } >) -> Result<(), embassy_rp::flash::Error> {
    let mut page = [0xFFu8; PAGE_SIZE];
    page[..16].copy_from_slice(&bytes);
    flash.erase(SETTINGS_ADDR, SETTINGS_ADDR + ERASE_SIZE as u32).await?;
    flash.write(SETTINGS_ADDR, &page).await
}

// CriticalSectionRawMutex is when data can be shared between threads and interrupts

// non-zero data is saved in flash memory and then copied to RAM, when increasing frames the storage will decrease twice as fast (goes to .data)
// todo: make animation frames zero at the start in static so they go to .bss and cost ~0 bytes of flash
pub static STATE: Mutex<CriticalSectionRawMutex, DeviceStorage> = Mutex::new(DeviceStorage::new());
static SETTINGS_DIRTY: AtomicBool = AtomicBool::new(false);
static ANIMATION_DIRTY: AtomicBool = AtomicBool::new(false);
static SAVE_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

pub fn request_settings_save()  { SETTINGS_DIRTY.store(true, Ordering::Relaxed);  SAVE_SIGNAL.signal(()); }
pub fn request_animation_save() { ANIMATION_DIRTY.store(true, Ordering::Relaxed); SAVE_SIGNAL.signal(()); }


#[embassy_executor::task]
pub async fn storage_task(
    mut flash: Flash<'static, embassy_rp::peripherals::FLASH, embassy_rp::flash::Async, FLASH_SIZE>
) {
    loop {
        SAVE_SIGNAL.wait().await;

        while with_timeout(Duration::from_millis(1000), SAVE_SIGNAL.wait()).await.is_ok() {}
        if SETTINGS_DIRTY.load(Ordering::Relaxed) {
            SETTINGS_DIRTY.store(false, Ordering::Relaxed);
            let bytes = STATE.lock().await.settings_bytes();
            if let Err(e) = save_settings(bytes, &mut flash).await { info!("settings save failed: {:?}", e); }
        }
        if ANIMATION_DIRTY.load(Ordering::Relaxed) {
            ANIMATION_DIRTY.store(false, Ordering::Relaxed);
            let state = STATE.lock().await;
            if let Err(e) = save_animation(&state, &mut flash).await { info!("animation save failed: {:?}", e); }
        }
    }
}