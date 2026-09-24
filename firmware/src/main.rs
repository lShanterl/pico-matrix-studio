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
mod storage;

use embassy_executor::Spawner;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use embedded_alloc::LlffHeap as Heap;
use matrix_protocol::Command;
use smart_leds::brightness;
use crate::storage::STATE;

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
    let mut mode = Mode::Live;

    loop {
        while let Ok(cmd) = COMMAND_CHANNEL.try_receive() {
            match cmd {
                Command::SetFrame(pixels) => {
                    matrix.set_frame(&pixels);
                    mode = Mode::Live;
                }
                Command::SetBrightness(b) => {
                    let safe_level = b.clamp(0, 100);
                    let brightness_f32 = safe_level as f32 / 100.0;
                    let mut storage = STATE.lock().await;
                    storage.brightness = brightness_f32;
                    matrix.request_refresh();
                }
                Command::SetAmper(mA) =>{
                    let mut storage = STATE.lock().await;
                    storage.max_amper = mA;
                    matrix.request_refresh();
                }
                Command::SelectAnimation(id) => {
                    mode = Mode::Procedural;
                    animation = animations::RotatingPlasmaAnimation;

                }
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
                let storage = STATE.lock().await;
                if storage.custom_animation.frame_count > 0 {
                    matrix.set_frame(&storage.custom_animation.frames[anim_idx % storage.custom_animation.frame_count]);
                    anim_idx += 1;
                    1000 / storage.custom_animation.fps as u64
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
