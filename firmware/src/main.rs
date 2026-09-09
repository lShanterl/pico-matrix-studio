#![no_std]
#![no_main]
extern crate alloc;

mod animations;
mod config;
mod irqs;
mod led_matrix;
mod panic;
mod tcp_listener;
mod usb;
mod wifi;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use embedded_alloc::LlffHeap as Heap;
use log::info;
use matrix_protocol::Command;
use crate::animations::STORED_ANIMATION;
use crate::Mode::Procedural;

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub static COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, Command, 4> = Channel::new();
enum Mode { Procedural, Live, Uploaded }

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = embassy_rp::init(Default::default());

    usb::init(spawner, peripherals.USB).await;

    let _wifi = wifi::init(
        spawner,
        wifi::WifiPeripherals {
            pio1: peripherals.PIO1,
            dma_ch1: peripherals.DMA_CH1,
            power_enable: peripherals.PIN_23,
            chip_select: peripherals.PIN_25,
            data: peripherals.PIN_24,
            clock: peripherals.PIN_29,
        },
    )
    .await;

    let mut matrix = led_matrix::LedMatrix::new(led_matrix::MatrixPeripherals {
        pio0: peripherals.PIO0,
        dma_ch0: peripherals.DMA_CH0,
        data_pin: peripherals.PIN_18,
    });

    let mut tick: u32 = 0;

    let mut anim_idx = 0;
    let mut animation = animations::RotatingPlasmaAnimation;
    let mut mode = Mode::Procedural;

    loop {
        if let Ok(cmd) = COMMAND_CHANNEL.try_receive() {
            match cmd {
                Command::SetFrame(pixels) => {
                    info!("set pico frame");
                    matrix.set_frame(&pixels);
                    mode = Mode::Live;
                }
                Command::SetBrightness(_b) => { /* apply scaling, unchanged mode */ }
                Command::SelectAnimation(_id) => { mode = Mode::Procedural; }
                Command::PlayUploadedAnimation => {
                    mode = Mode::Uploaded;
                    anim_idx = 0;
                }
                _ => {} // upload-related tags never reach here
            }
        }

        let delay_ms = match mode {
            Mode::Procedural => {
                matrix.set_frame(&animation.next_frame(tick));
                tick += 1;
                16 // fixed 60fps for procedural animations
            }
            Mode::Live => {
                16 // keep last frame
            }
            Mode::Uploaded => {
                let anim = STORED_ANIMATION.lock().await;
                if anim.frame_count > 0 {
                    matrix.set_frame(&anim.frames[anim_idx % anim.frame_count]);
                    anim_idx += 1;
                    1000 / anim.fps as u64
                } else {
                    mode = Mode::Procedural; // nothing stored yet, fall back
                    16
                }
            }
        };
        matrix.show().await;
        Timer::after(Duration::from_millis(delay_ms)).await;
    }
}
