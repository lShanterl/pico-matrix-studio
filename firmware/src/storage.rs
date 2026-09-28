use crc::{Crc, CRC_32_ISO_HDLC};
use embassy_rp::flash::{Async, Flash};
use embassy_rp::peripherals::FLASH;
use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embedded_storage_async::nor_flash::NorFlash;
use log::info;
use crate::animations::StoredAnimation;

pub const ADDR_OFFSET: u32 = 0x100000;
pub const FLASH_SIZE: usize = 2 * 1024 * 1024;
pub const ERASE_SIZE: usize = 4096;
pub const PAGE_SIZE: usize = 256;
const STORAGE_VERSION: u32 = 3; // a safety measure to avoid memory corruption - incremented by one whenever data inside device storage changes

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
    pub version: u32,
    pub brightness: f32,
    pub max_amper: u16,
    pub current_animation: u8,
    pub mode: u8,
    pub custom_animation: StoredAnimation,
    pub crc: u32 // control sum to avoid data corruption when device is unplugged during save / load
}

impl DeviceStorage {

    pub fn mode(&self) -> Mode { Mode::from_u8(self.mode) }
    pub fn set_mode(&mut self, m: Mode) { self.mode = m as u8; }
    fn calculate_crc(&self) -> u32 {
        let bytes = self.as_bytes();
        // decreased by u32 size (crc)
        let data_len = core::mem::size_of::<DeviceStorage>() - 4usize;

        let data_to_hash = &bytes[..data_len];

        CRC_CALCULATOR.checksum(data_to_hash)
    }

    fn as_bytes(&self) -> &[u8]{
        let size = core::mem::size_of::<Self>();
        unsafe {
            core::slice::from_raw_parts(
                (self as *const DeviceStorage) as *const u8,
                size,
            )
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        assert_eq!(bytes.len(), core::mem::size_of::<DeviceStorage>());
        let mut storage = Self::new();
        unsafe {
            let storage_ptr = (&mut storage as *mut DeviceStorage) as *mut u8;
            core::ptr::copy_nonoverlapping(bytes.as_ptr(), storage_ptr, bytes.len());
        }
        storage
    }

    pub async fn load_from_flash(
        &mut self,
        flash: &mut Flash<'_, FLASH, Async, FLASH_SIZE>,
    ) {
        let bytes = unsafe {
            core::slice::from_raw_parts_mut(self as *mut Self as *mut u8, core::mem::size_of::<Self>())
        };
        let ok = flash.read(ADDR_OFFSET, bytes).await.is_ok()
            && self.version == STORAGE_VERSION
            && self.crc == self.calculate_crc();
        if !ok {
            *self = Self::new();
        }
    }

    //todo: potential improvement - split the settings part & animation frames into separate sections
    pub async fn save_to_flash(
        &mut self,
        flash: &mut Flash<'_, FLASH,embassy_rp::flash::Async, { FLASH_SIZE } >
    ) -> Result<(), embassy_rp::flash::Error>{
        let size = core::mem::size_of::<DeviceStorage>();
        self.crc = self.calculate_crc();
        let data = self.as_bytes();

        // sector is the smallest number of bytes that can be deleted 4096
        let sectors_to_erase = (size + ERASE_SIZE - 1) / ERASE_SIZE;
        let erase_bytes = (sectors_to_erase * ERASE_SIZE) as u32;

        flash.erase(ADDR_OFFSET, ADDR_OFFSET + erase_bytes).await?;

        // rounded down bytes to a multiple of PAGE_SIZE
        let full = size - size % PAGE_SIZE;
        flash.write(ADDR_OFFSET, &data[..full]).await?;
        if full < size {
            let mut last = [0xFFu8; PAGE_SIZE];
            last[..size - full].copy_from_slice(&data[full..]);
            flash.write(ADDR_OFFSET + full as u32, &last).await?;
        }

        Ok(())
    }

    pub const fn new() -> Self {

        Self {
            version: STORAGE_VERSION,
            brightness: 1.0f32,    // default to max brightness
            max_amper: 500,        // default to 500mA
            current_animation: 0,
            mode: 0,
            custom_animation: StoredAnimation::new(),
            crc: 0,
        }
    }
}

// CriticalSectionRawMutex is when data can be shared between threads and interrupts

// non-zero data is saved in flash memory and then copied to RAM, when increasing frames the storage will decrease twice as fast (goes to .data)
// todo: make animation frames zero at the start in static so they go to .bss and cost ~0 bytes of flash
pub static STATE: Mutex<CriticalSectionRawMutex, DeviceStorage> = Mutex::new(DeviceStorage::new());
pub static SAVE_CHANNEL: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();

#[embassy_executor::task]
pub async fn storage_task(
    mut flash: Flash<'static, embassy_rp::peripherals::FLASH, embassy_rp::flash::Async, FLASH_SIZE>
) {
    loop {
        SAVE_CHANNEL.receive().await;
        while embassy_time::with_timeout(
            embassy_time::Duration::from_millis(1000),
            SAVE_CHANNEL.receive(),
        )
            .await
            .is_ok()
        {}

        let mut state = STATE.lock().await;
        if let Err(e) = state.save_to_flash(&mut flash).await {
            info!("Failed to save flash: {:?}", e);
        }
    }
}