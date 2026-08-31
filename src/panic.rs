use core::panic::PanicInfo;
use log::error;

// Firmware-wide panic handler, independent of whether usb-debug is enabled
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{}", info); // is a no-op if no logger has been installed (e.g. disabled usb-debug feature)

    // Spin loop. If USB processing runs on interrupts or DMA,
    // keeping execution alive gives the serial buffer time to flush out
    loop {
        core::hint::spin_loop();
    }
}