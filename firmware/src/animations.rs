use matrix_protocol::{Frame, MATRIX_HEIGHT, MATRIX_PIXEL_COUNT, MATRIX_WIDTH, MAX_ANIMATION_FRAMES};
use crate::led_matrix::{xy_to_index};
use embassy_rp::rom_data::float_funcs::{fcos, fsin, fsqrt};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use smart_leds::RGB8;
use smart_leds::hsv::{Hsv, hsv2rgb};


pub trait Animation {
    fn next_frame(&mut self, tick: u32) -> Frame;
}

// for non-calculative compile-time animations
pub struct StaticAnimation {
    frames: &'static [Frame],
}
impl StaticAnimation {
    pub fn new(frames: &'static [Frame]) -> Self {
        Self { frames }
    }
}

impl Animation for StaticAnimation {
    // Might need to change the logic or reset ticks when animation changes
    fn next_frame(&mut self, tick: u32) -> Frame {
        self.frames[(tick as usize) % self.frames.len()]
    }
}

pub struct RotatingPlasmaAnimation;
impl RotatingPlasmaAnimation {
    pub fn next_frame(&mut self, tick: u32) -> Frame {
        let mut frame = [RGB8::default(); MATRIX_PIXEL_COUNT];
        let t = (tick % 10000) as f32 * 0.02;

        for x in 0..MATRIX_WIDTH {
            for y in 0..MATRIX_HEIGHT {
                let cx = x as f32 - 7.5;
                let cy = y as f32 - 7.5;
                let r = fsqrt(cx * cx + cy * cy);

                let v1 = fsin(fcos(x as f32 * 0.3) * 0.5 + t * -0.33);

                let v2 = fsin(fcos(y as f32) * 0.1 + t * 0.9);

                let v3 = fsin((x as f32 * 0.15 + y as f32 * 0.33) * 0.4 - t * 0.5);

                let v4 = fsin(r * 0.3 - t * 2.0);

                let total = v1 + v2 + v3 + v4;
                let normalized = ((total + 4.0) / 8.0).clamp(0.0, 1.0);

                let hue = ((normalized * 255.0 * 3.0) % 256.0) as u8; // denser color change * 3
                let sat: u8 = 255;
                let val: u8 = 25;
                let hsv: Hsv = Hsv { hue, sat, val };

                frame[xy_to_index(x, y)] = hsv2rgb(hsv);
            }
        }

        frame
    }
}

pub struct StoredAnimation {
    pub frames: [Frame; MAX_ANIMATION_FRAMES],
    pub frame_count: usize,
    pub fps: u8,
}

impl StoredAnimation {
    pub const fn new() -> Self {
        Self {
            frames: [[RGB8::new(0, 0, 0); MATRIX_PIXEL_COUNT]; MAX_ANIMATION_FRAMES],
            frame_count: 0,
            fps: 30,
        }
    }
}

// CriticalSectionRawMutex is when data can be shared between threads and interrupts
pub static STORED_ANIMATION: Mutex<CriticalSectionRawMutex, StoredAnimation> = Mutex::new(StoredAnimation::new());
