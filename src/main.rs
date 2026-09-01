#![no_std]
#![no_main]

mod config;
mod irqs;
mod usb;
mod wifi;
mod led_matrix;
mod panic;
mod animations;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

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

    let mut tick: u32 = 0;

    let mut animation = animations::RotatingPlasmaAnimation;

    loop {
        let frame = animation.next_frame(tick);
        matrix.set_frame(&frame);
        matrix.show().await;

        tick += 1;

        Timer::after(Duration::from_millis(16)).await; // for 60fps
    }
}