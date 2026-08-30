use cyw43::JoinOptions;
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_hal_internal::Peri;
use embassy_rp::dma;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIN_23, PIN_24, PIN_25, PIN_29, PIO1};
use embassy_rp::pio::Pio;
use log::info;
use static_cell::StaticCell;
use crate::config::{WIFI_PASSWORD, WIFI_SSID};
use crate::irqs::Irqs;

static CYW43_STATE: StaticCell<cyw43::State> = StaticCell::new();
static WIFI_FIRMWARE: aligned::Aligned<aligned::A4, [u8; include_bytes!("../firmware/43439A0.bin").len()]> =
    aligned::Aligned(*include_bytes!("../firmware/43439A0.bin"));

// Nvram - saves configuration data even after the device disconnects
static NVRAM: aligned::Aligned<aligned::A4, [u8; include_bytes!("../firmware/nvram_rp2040.bin").len()]> =
    aligned::Aligned(*include_bytes!("../firmware/nvram_rp2040.bin"));

/*
    Układ Wi-Fi CYW43439 zamontowany na Raspberry Pi Pico W jest osobnym procesorem.
    Aby mikrokontroler RP2040 mógł z nim współpracować, musi stale odbierać z niego powiadomienia sprzętowe
    utrzymywać połączenie radiowe z routerem (obsługiwać tzw. pakiety keep-alive, re-negocjacje kluczy szyfrujących)
    oraz przesyłać bajty przez interfejs PIO/DMA. wifi_task to bezkresna pętla, która się tym zajmuje.
 */

// Gdyby nie te zadania w tle, musiałbyś w swojej pętli loop co kilka milisekund ręcznie wywoływać funkcje typu cyw43_poll() oraz net_stack_poll()
// co drastycznie skomplikowałoby kod i prowadziłoby do gubienia pakietów.

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO1, 0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut stack: embassy_net::Runner<'static, cyw43::NetDriver<'static>>
) -> ! {
    stack.run().await
}

pub struct WifiPeripherals {
    pub pio1: Peri<'static, PIO1>,
    pub dma_ch1: Peri<'static, DMA_CH1>,
    pub power_enable: Peri<'static, PIN_23>,
    pub chip_select: Peri<'static, PIN_25>,
    pub data: Peri<'static, PIN_24>,
    pub clock: Peri<'static, PIN_29>,
}

#[allow(dead_code)]
pub struct Wifi {
    pub control: cyw43::Control<'static>,
    pub net_device: cyw43::NetDriver<'static>,
}

pub async fn init(spawner : Spawner, p: WifiPeripherals) -> Wifi {
    let country_locale_matrix = include_bytes!("../firmware/43439A0_clm.bin");

    // Configuring physical pins that manage network device on Raspberry Pico W (must be those pins)
    let wifi_power_enable_pin = Output::new(p.power_enable, Level::Low);
    let wifi_chip_select_pin = Output::new(p.chip_select, Level::High);


    // Initializing PIO1 block for wireless communication (PIO0 reserved for the WS2812)
    let mut pio1_hardware_block = Pio::new(p.pio1, Irqs);
    let dma_channel = dma::Channel::new(p.dma_ch1, Irqs);

    let pio_spi_bus = PioSpi::new(
        &mut pio1_hardware_block.common,
        pio1_hardware_block.sm0,
        cyw43_pio::DEFAULT_CLOCK_DIVIDER,
        pio1_hardware_block.irq0,
        wifi_chip_select_pin,
        p.data, // Data pin
        p.clock, // Clock pin
        dma_channel, // Dedicated DMA channel for fast memory transfer
    );

    let cyw43_state = CYW43_STATE.init(cyw43::State::new());

    // Virtual network card, sending commands to Wi-Fi, engine for sending data
    let (net_device, mut control, runner) = cyw43::new(cyw43_state, wifi_power_enable_pin, pio_spi_bus, &WIFI_FIRMWARE, &NVRAM).await;

    spawner.spawn(wifi_task(runner).unwrap());

    control.init(country_locale_matrix).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    info!("Connecting to Wi-Fi: {}", WIFI_SSID);
    match control
        .join(WIFI_SSID, JoinOptions::new(WIFI_PASSWORD.as_bytes()))
        .await
    {
        Ok(_) => info!("Connected to Wi-Fi!"),
        Err(err) => info!("Error connecting to Wi-Fi: status={:?}", err),
    }

    Wifi { control, net_device }
}