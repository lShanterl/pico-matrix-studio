use crate::irqs::Irqs;
use alloc::vec::Vec;
use cyw43::{JoinOptions, ScanOptions};
use cyw43_pio::PioSpi;
use embassy_executor::Spawner;
use embassy_hal_internal::Peri;
use embassy_net::udp::PacketMetadata;
use embassy_net::{Config, Ipv4Address, Ipv4Cidr, StackResources, StaticConfigV4};
use embassy_rp::dma;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{DMA_CH0, DMA_CH1, PIN_23, PIN_24, PIN_25, PIN_29, PIO1};
use embassy_rp::pio::Pio;
use embassy_time::{with_timeout, Duration};
use log::info;
use static_cell::StaticCell;
use crate::config::{WIFI_PASSWORD, WIFI_SSID};
use crate::tcp_listener::control_task;

static CYW43_STATE: StaticCell<cyw43::State> = StaticCell::new();
static WIFI_FIRMWARE: aligned::Aligned<
    aligned::A4,
    [u8; include_bytes!("../wifi-blobs/43439A0.bin").len()],
> = aligned::Aligned(*include_bytes!("../wifi-blobs/43439A0.bin"));

// Nvram - saves configuration data even after the device disconnects
static NVRAM: aligned::Aligned<
    aligned::A4,
    [u8; include_bytes!("../wifi-blobs/nvram_rp2040.bin").len()],
> = aligned::Aligned(*include_bytes!("../wifi-blobs/nvram_rp2040.bin"));

// CYW43439 mounted on pico is a standalone processor on Pico W, in order to work with the RP2040 microcontroller it must constantly cooperate with it (wifi_task is responsible for that)
// If not for those background tasks it would be necesary to call fns such as cyw43_poll() or net_stack_poll() that would lead to packet loss

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<'static, cyw43::SpiBus<Output<'static>, PioSpi<'static, PIO1, 0>>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut stack: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
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

static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();

pub async fn init(spawner: Spawner, p: WifiPeripherals) -> embassy_net::Stack<'static> {
    let country_locale_matrix = include_bytes!("../wifi-blobs/43439A0_clm.bin");

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
        p.data,      // Data pin
        p.clock,     // Clock pin
        dma_channel, // Dedicated DMA channel for fast memory transfer
    );

    let cyw43_state = CYW43_STATE.init(cyw43::State::new());

    // Virtual network card, sending commands to Wi-Fi, engine for sending data
    let (net_device, mut control, runner) = cyw43::new(
        cyw43_state,
        wifi_power_enable_pin,
        pio_spi_bus,
        &WIFI_FIRMWARE,
        &NVRAM,
    )
    .await;

    spawner.spawn(wifi_task(runner).unwrap());

    control.init(country_locale_matrix).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    info!("Scanning for nearby Wi-Fi networks...");
    let mut scanner = control.scan(ScanOptions::default()).await;
    while let Some(bss) = scanner.next().await {
        if let Ok(ssid) = core::str::from_utf8(&bss.ssid[..bss.ssid_len as usize]) {
            info!("  seen: '{}' rssi={} channel={}", ssid, bss.rssi, bss.chanspec);
        }
    }
    drop(scanner);
    
    info!("Connecting to Wi-Fi: {}", WIFI_SSID);
    match with_timeout(
        Duration::from_secs(15),
        control.join(WIFI_SSID, JoinOptions::new(WIFI_PASSWORD.as_bytes())),
    )
        .await
    {
        Ok(Ok(_)) => info!("Connected to Wi-Fi!"),
        Ok(Err(err)) => info!("Error connecting to Wi-Fi: status={:?}", err),
        Err(_) => info!("Join timed out after 15s (SSID not found, or auth failing silently)"),
    }

    info!("after connecting to Wi-Fi");

    let config = Config::ipv4_static(StaticConfigV4 {
        address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 1, 50), 24),
        gateway: Some(Ipv4Address::new(192, 168, 1, 1)),
        dns_servers: Default::default(),
    });

    let (stack, runner) = embassy_net::new(
        net_device,
        config,
        RESOURCES.init(StackResources::new()),
        0x1234_5678,
    );

    spawner.spawn(net_task(runner).unwrap());

    spawner.spawn(control_task(stack).unwrap());

    stack
}
