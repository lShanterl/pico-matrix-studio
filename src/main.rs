#![no_std]
#![no_main]

mod config;
mod irqs;
mod usb;
mod wifi;
mod led_matrix;
mod panic;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use smart_leds::{SmartLedsWrite, RGB8};
use crate::config::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = embassy_rp::init(Default::default());

    usb::init(spawner, peripherals.USB).await;

    let wifi = wifi::init(spawner, wifi::WifiPeripherals {
        pio1: peripherals.PIO1,
        dma_ch1: peripherals.DMA_CH1,
        power_enable: peripherals.PIN_23,
        chip_select: peripherals.PIN_25,
        data: peripherals.PIN_24,
        clock: peripherals.PIN_29,
    }).await;

    let mut matrix = led_matrix::LedMatrix::new(led_matrix::MatrixPeripherals {
        pio0: peripherals.PIO0,
        dma_ch0: peripherals.DMA_CH0,
        data_pin: peripherals.PIN_18,
    });

    for y in 0..16 {
        for x in 0..16 {
            matrix.set(x, y, if HEART_MAP[y][x] == 1 { RGB8::new(22, 14, 25) } else { RGB8::default() });
        }
    }

    loop {
        Timer::after(Duration::from_secs(2)).await;

        matrix.show().await;
    }
}