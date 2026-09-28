use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, PIO0, PIO1, USB};
use embassy_rp::pio::InterruptHandler as PioInterruptHandler;

// Binding the peripheral interrupt vectors to the embassy handlers used by the driver

embassy_rp::bind_interrupts!(pub struct Irqs {
    PIO1_IRQ_0 => PioInterruptHandler<PIO1>;
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    // led, wi-fi, flash
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>, embassy_rp::dma::InterruptHandler<DMA_CH2>;
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});
