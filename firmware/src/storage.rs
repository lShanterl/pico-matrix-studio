use embassy_sync::mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use crate::animations::StoredAnimation;

pub struct DeviceStorage {
    pub brightness: f32,
    pub max_amper: u16,
    pub current_animation: u8,
    pub custom_animation: StoredAnimation,
}

impl DeviceStorage {
    pub const fn new() -> Self {
        Self {
            brightness: 1.0f32,    // default to max brightness
            max_amper: 500,        // default to 500mA
            current_animation: 0,
            custom_animation: StoredAnimation::new(),
        }
    }
}

// CriticalSectionRawMutex is when data can be shared between threads and interrupts
pub static STATE: Mutex<CriticalSectionRawMutex, DeviceStorage> = Mutex::new(DeviceStorage::new());