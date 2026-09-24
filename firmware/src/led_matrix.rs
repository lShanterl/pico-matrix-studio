use matrix_protocol::{estimate_current_ma, Frame, MATRIX_HEIGHT, MATRIX_PIXEL_COUNT, MATRIX_WIDTH, MAX_CURRENT_MA};
use crate::irqs::Irqs;
use embassy_hal_internal::Peri;
use embassy_rp::gpio::AnyPin;
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIN_18, PIO0};
use embassy_rp::pio::{Pio, PioPin};
use embassy_rp::pio_programs::ws2812::{Grb, PioWs2812, PioWs2812Program, Rgb, RgbColorOrder, RgbwPioWs2812};
use log::info;
use smart_leds::RGB8;
use crate::storage::STATE;

// Calculates coordinates (x, y) to linear index of pixels buffer. The matrix is wired in a zig-zag pattern, so even rows are left-to-right and odd rows are right-to-left
pub fn xy_to_index(x: usize, y: usize) -> usize {
    if y % 2 == 0 {
        y * MATRIX_WIDTH + x
    } else {
        y * MATRIX_WIDTH + (MATRIX_WIDTH - 1 - x)
    }
}

pub struct MatrixPeripherals<P: PioPin> {
    pub pio0: Peri<'static, PIO0>,
    pub dma_ch0: Peri<'static, DMA_CH0>,
    pub data_pin: Peri<'static, P>,
}

pub struct LedMatrix {
    driver: PioWs2812<'static, PIO0, 0, MATRIX_PIXEL_COUNT, Grb>,
    pixels: [RGB8; MATRIX_PIXEL_COUNT],
    should_refresh: bool, //optimization - once a frame is set, nothing changes until the next as WS2812 doesn't need refreshing like a display it just holds the last state
}

impl LedMatrix {
    pub fn new<P: PioPin>(p: MatrixPeripherals<P>) -> Self {
        let Pio {
            mut common, sm0, ..
        } = Pio::new(p.pio0, Irqs);

        let program = PioWs2812Program::new(&mut common);

        let driver: PioWs2812<'static, PIO0, 0, MATRIX_PIXEL_COUNT, Grb> =
            PioWs2812::new(
                &mut common,
                sm0,
                p.dma_ch0,
                Irqs,
                p.data_pin,
                &program,
            );

        Self {
            driver,
            pixels: [RGB8::default(); MATRIX_PIXEL_COUNT],
            should_refresh: true,
        }
    }

    pub fn clear(&mut self) {
        self.pixels = [RGB8::default(); MATRIX_PIXEL_COUNT];
        self.should_refresh = true;

    }

    pub fn set(&mut self, x: usize, y: usize, color: RGB8) {
        self.pixels[xy_to_index(x, y)] = color;
        self.should_refresh = true;
    }

    pub fn set_color(&mut self, color: RGB8) {
        self.pixels = [color; MATRIX_PIXEL_COUNT];
        self.should_refresh = true;
    }

    pub fn set_pixels(&mut self, pixels: [RGB8; MATRIX_PIXEL_COUNT]) {
        self.pixels = pixels;
        self.should_refresh = true;
    }

    pub fn set_frame(&mut self, frame: &Frame) {
        for y in 0..MATRIX_HEIGHT {
            for x in 0..MATRIX_WIDTH {
                self.pixels[xy_to_index(x, y)] = frame[y * MATRIX_WIDTH + x];
            }
        }
        self.should_refresh = true;
    }

    pub async fn show(&mut self) {
        if(!self.should_refresh) {
            return;
        }

        let mut corrected = self.pixels;
        matrix_protocol::gamma_correct(&mut corrected);

        // dropping the state immediately to avoid waiting for the frame to be written
        let (brightness, max_ma) = {
            let s = STATE.lock().await;
            (s.brightness, s.max_amper)
        };

        let report = matrix_protocol::apply_brightness_limited(&mut corrected, brightness, max_ma);


        self.driver.write(&corrected).await;
        self.should_refresh = false;
    }

    pub fn request_refresh(&mut self) {
        self.should_refresh = true;
    }
}
