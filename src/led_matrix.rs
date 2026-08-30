use embassy_hal_internal::Peri;
use embassy_rp::gpio::AnyPin;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIN_18, PIO0};
use embassy_rp::pio::{Pio, PioPin};
use embassy_rp::pio_programs::ws2812::{PioWs2812, PioWs2812Program, Rgb, RgbColorOrder, RgbwPioWs2812};
use log::info;
use smart_leds::RGB8;
use crate::config::{MATRIX_PIXEL_COUNT, MATRIX_WIDTH, MAX_CURRENT_MA};
use crate::irqs::Irqs;

// Calculates coordinates (x, y) to linear index of pixels buffer. The matrix is wired in a zig-zag pattern, so even rows are left-to-right and odd rows are right-to-left
fn xy_to_index(x: usize, y: usize) -> usize {
    if y % 2 == 0 {
        y * MATRIX_WIDTH + x
    } else {
        y * MATRIX_WIDTH + (MATRIX_WIDTH - 1 - x)
    }
}

// Estimates the current consumption of the LED matrix in milliamperes based on the pixel colors. Used as precaution in order not to burn the charger
fn estimate_current_ma(pixels: &[RGB8]) -> f32 {
    let idle_current_ma = pixels.len() as f32;
    let mut total_channel_sum = 0u32;

    for p in pixels{
        total_channel_sum += p.r as u32 + p.g as u32 + p.b as u32;
    }
    let leds_current_ma = (total_channel_sum as f32 * 20f32) / 255f32;
    leds_current_ma as f32 + idle_current_ma as f32
}

pub struct MatrixPeripherals<P: PioPin> {
    pub pio0: Peri<'static, PIO0>,
    pub dma_ch0: Peri<'static, DMA_CH0>,
    pub data_pin: Peri<'static, P>,
}

pub struct LedMatrix {
    driver: PioWs2812<'static, PIO0, 0, MATRIX_PIXEL_COUNT, Rgb>,
    pixels: [RGB8; MATRIX_PIXEL_COUNT],
}

impl LedMatrix {
    pub fn new<P: PioPin>(p: MatrixPeripherals<P>) -> Self {
        let Pio {
            mut common, sm0, ..
        } = Pio::new(p.pio0, Irqs);

        let program = PioWs2812Program::new(&mut common);

        let driver: PioWs2812<'static, PIO0, 0, MATRIX_PIXEL_COUNT, Rgb> = PioWs2812::<PIO0, 0, MATRIX_PIXEL_COUNT, Rgb>::with_color_order(
            &mut common,
            sm0,
            p.dma_ch0,
            Irqs,
            p.data_pin,
            &program,
        );

        Self{
            driver,
            pixels: [RGB8::default(); MATRIX_PIXEL_COUNT],
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [RGB8::default(); MATRIX_PIXEL_COUNT];
    }

    pub fn set(&mut self, x: usize, y: usize, color: RGB8) {
        self.pixels[xy_to_index(x, y)] = color;
    }

    pub fn set_color(&mut self, color: RGB8) {
        self.pixels = [color; MATRIX_PIXEL_COUNT];
    }

    pub fn set_pixels(&mut self, pixels: [RGB8; MATRIX_PIXEL_COUNT]){
        self.pixels = pixels;
    }

    pub fn estimate_current_ma(&self) -> f32 {
        estimate_current_ma(&self.pixels)
    }

    pub async fn show(&mut self) {
        let current = self.estimate_current_ma();
        if current > MAX_CURRENT_MA as f32 {
            info!(
                "Skipped current frame: estimated amperage: {} mA is higher than {} mA limit",
                current, MAX_CURRENT_MA
            );
            return;
        }
        self.driver.write(&self.pixels).await;
    }
}