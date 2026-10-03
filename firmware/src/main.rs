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
mod bootsel;
mod beacon;

use embassy_executor::Spawner;
use embassy_rp::flash::Flash;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use embedded_alloc::LlffHeap as Heap;
use matrix_protocol::Command;
use crate::irqs::Irqs;
use crate::storage::{storage_task, Mode, STATE};

#[global_allocator]
static HEAP: Heap = Heap::empty();

pub static COMMAND_CHANNEL: Channel<CriticalSectionRawMutex, Command, 4> = Channel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let peripherals = embassy_rp::init(Default::default());

    usb::init(spawner, peripherals.USB).await;
    spawner.spawn(wifi::wifi_manager(
        spawner,
        wifi::WifiPeripherals {
            pio1: peripherals.PIO1,
            dma_ch1: peripherals.DMA_CH1,
            power_enable: peripherals.PIN_23,
            chip_select: peripherals.PIN_25,
            data: peripherals.PIN_24,
            clock: peripherals.PIN_29,
        },
    ).unwrap());

    let mut matrix = led_matrix::LedMatrix::new(led_matrix::MatrixPeripherals {
        pio0: peripherals.PIO0,
        dma_ch0: peripherals.DMA_CH0,
        data_pin: peripherals.PIN_18,
    });

    let mut flash: Flash<'_, _, _, { storage::FLASH_SIZE }> = embassy_rp::flash::Flash::new(peripherals.FLASH, peripherals.DMA_CH2, Irqs);

    let saved_mode = {
        let mut state = STATE.lock().await;
        state.load_from_flash(&mut flash).await;
        state.mode()
    };

    spawner.spawn(storage_task(flash).unwrap());

    let mut tick: u32 = 0;
    let mut anim_idx: usize = 0;
    let mut animation = animations::RotatingPlasmaAnimation;
    let mut mode = saved_mode;

    loop {
        while let Ok(cmd) = COMMAND_CHANNEL.try_receive() {
            match cmd {
                Command::SetFrame(pixels) => {
                    // a single frame is stored as a one-frame animation
                    {
                        let mut s = STATE.lock().await;
                        s.custom_animation.frames[0] = pixels;
                        s.custom_animation.frame_count = 1;
                        s.set_mode(Mode::Uploaded);
                    }
                    mode = Mode::Uploaded;
                    anim_idx = 0;
                    storage::request_animation_save();
                    storage::request_settings_save();
                }
                Command::SetBrightness(b) => {
                    let safe_level = b.clamp(0, 100);
                    let brightness_f32 = safe_level as f32 / 100.0;
                    {
                        let mut storage = STATE.lock().await;
                        storage.brightness = brightness_f32;
                    }
                    storage::request_settings_save();

                    matrix.request_refresh();
                }
                Command::SetAmper(mA) =>{
                    {
                        let mut storage = STATE.lock().await;
                        storage.max_amper = mA;
                    }
                    storage::request_settings_save();
                    matrix.request_refresh();
                }
                Command::SelectAnimation(id) => {
                    mode = Mode::Procedural;
                    animation = animations::RotatingPlasmaAnimation; // todo: pick by id

                }
                Command::PlayUploadedAnimation => {
                    mode = Mode::Uploaded;
                    anim_idx = 0;
                    STATE.lock().await.set_mode(Mode::Uploaded);
                    storage::request_settings_save();
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
            Mode::Uploaded => {
                let s = STATE.lock().await;
                let count = s.custom_animation.frame_count;
                if count == 0 {
                    mode = Mode::Procedural; // nothing stored, fall back
                    16
                } else if count == 1 {
                    if anim_idx == 0 {
                        matrix.set_frame(&s.custom_animation.frames[0]);
                        anim_idx = 1;          // drawn, don't touch the matrix again
                    }
                    16                         // show() returns early, so this is nearly free
                } else {
                    matrix.set_frame(&s.custom_animation.frames[anim_idx % count]);
                    anim_idx += 1;
                    1000 / s.custom_animation.fps.max(1) as u64
                }
            }
        };
        matrix.show().await;
        Timer::after(Duration::from_millis(delay_ms)).await;
    }
}
