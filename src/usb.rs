use embassy_executor::Spawner;
use embassy_hal_internal::Peri;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::UsbDevice;
use log::{info};
use static_cell::StaticCell;
use crate::irqs::Irqs;

#[allow(dead_code)]
#[allow(unused_imports)]
#[allow(unused_variables)]

// Debug-only USB CDC logger used for development logging and serial output.
// This module is only for debugging and should not be used in deploy builds.

static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();
static USB_STATE: StaticCell<embassy_usb::class::cdc_acm::State> = StaticCell::new();

#[embassy_executor::task]
async fn usb_task(mut device: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    device.run().await
}

#[embassy_executor::task]
async fn logger_task(class: CdcAcmClass<'static, Driver<'static, USB>>) -> ! {
    embassy_usb_logger::with_class!(1024, log::LevelFilter::Info, class).await
}

// Builds cdc device and waits till terminal (such as PuTTY) connects to the device via DTR signal.
// After returning from this function log::info!() calls will be printed to the terminal.
pub async fn init(spawner: Spawner, usb: Peri<'static, USB>) {
    let driver = Driver::new(usb, Irqs);

    // usb configuration
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Raspberry Pi");
    config.product = Some("Pico W CDC");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    // usb structure & cdc acm - Communications Device Class Abstract Control Model
    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init([0; 256]),
        BOS_DESCRIPTOR.init([0; 256]),
        &mut [], // msos descriptors
        CONTROL_BUF.init([0; 64]),
    );


    let state = USB_STATE.init(embassy_usb::class::cdc_acm::State::new());
    let mut class = CdcAcmClass::new(&mut builder, state, 64);


    let usb = builder.build();
    spawner.spawn(usb_task(usb).unwrap());

    // Waiting until a terminal connects
    #[cfg(feature = "usb-debug")]
    while !class.dtr() {
        Timer::after(Duration::from_millis(100)).await;
    }

    // Short wait to make sure everything will be printed correctly
    #[cfg(feature = "usb-debug")]
    Timer::after(Duration::from_millis(200)).await;

    spawner.spawn(logger_task(class).unwrap());

    info!("--- Terminal is connected! Starting further configuration of Pico W ---"); // logging is effectively free when nobody's listening. There's no runtime reason to strip it
}