// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bytemuck::{Pod, Zeroable};
use matrix_protocol::{Command, Frame, LEN_PREFIX_BYTES, MATRIX_PIXEL_COUNT, MAX_COMMAND_BYTES, MAX_WIRE_BYTES, PORT};
use serde::Deserialize;
use serialport::SerialPort;
use smart_leds::{ RGB8};
use std::io::Write;
use tokio::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;


struct NetworkState(Mutex<Option<TcpStream>>);

#[derive(Clone, serde::Serialize)]
struct ConnectionStatus {
    connected: bool,
    ip: Option<String>,
    error: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct PowerEstimate { current_ma: f32, max_current_ma: u32, over_limit: bool }

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

fn to_frame(pixels: &[RGB]) -> Result<Frame, String> {
    if pixels.len() != MATRIX_PIXEL_COUNT {
        return Err("Layout size does not match MATRIX_PIXEL_COUNT".into());
    }
    let mut frame = [RGB8::default(); MATRIX_PIXEL_COUNT];
    for (dst, src) in frame.iter_mut().zip(pixels) {
        *dst = RGB8 { r: src.r, g: src.g, b: src.b };
    }
    Ok(frame)
}

#[tauri::command]
async fn send_frame_to_pico(state: tauri::State<'_,NetworkState>, frames: Vec<Vec<RGB>>, fps: u32) -> Result<(), String> {

    // convert to smart-leds RGB8 first, before connection to ensure safety
    let frames = frames
        .iter()
        .map(|f| to_frame(f))
        .collect::<Result<Vec<Frame>, _>>()?;

    //todo: verify frames length - need to take into the account the spare memory on pico  & whether the frames can be longer than 255

    let mut guard = state.0.lock().await;

    let stream = guard.as_mut().ok_or("Not connected")?;

    match frames.len(){
        0=> Ok(()),
        1=> {
            write_command(stream,&Command::SetFrame(frames[0])).await
        },
        n=>{
            write_command(stream, &Command::UploadAnimationStart {
                frame_count: n as u8,
                fps: fps as u8,
            }).await?;

            for (i, frame) in frames.into_iter().enumerate() {
                write_command(stream, &Command::UploadAnimationFrame {
                    index: i as u8,
                    frame,
                }).await?;
            }

            write_command(stream, &Command::UploadAnimationEnd).await?;
            write_command(stream, &Command::PlayUploadedAnimation).await
        }
    }
}

#[tauri::command]
async fn connect_to_pico(app: AppHandle, state: tauri::State<'_, NetworkState>, ip: String) -> Result<(), String> {
    println!("Connecting to pico...");
    let mut guard = state.0.lock().await;
    let addr = format!("{}:{}", ip, PORT);

    return match TcpStream::connect(addr).await {
        Ok(stream) => {
            stream.set_nodelay(true).unwrap();

            *guard = Some(stream);

            app.emit(
                "pico-connection-status",
                ConnectionStatus {
                    connected: true,
                    ip: Some(ip),
                    error: None,
                },
            )
                .ok();
            Ok(())
        }
        Err(er) => {
            app.emit(
                "pico-connection-status",
                ConnectionStatus {
                    connected: false,
                    ip: None,
                    error: Some(er.to_string()),
                },
            )
                .ok();
            println!("Failed to connect to pico: {}", er);
            Err("Failed to connect to pico".to_owned())
        }
    }
}

#[tauri::command]
async fn disconnect_from_pico(app: AppHandle, state: tauri::State<'_,NetworkState>) -> Result<(), String> {
    let mut guard = state.0.lock().await;

    if let Some(mut stream) = guard.take() {
        if let Err(e) = stream.shutdown().await {
            eprintln!("Failed to shutdown stream: {}", e);
        }

        app.emit(
            "pico-connection-status",
            ConnectionStatus {
                connected: false,
                ip: None,
                error: None,
            },
        ).ok();

        println!("Disconnected from pico");
        Ok(())
    } else {
        Err(String::from("Failed to disconnect: not connected"))
    }
}
#[tauri::command]
fn estimate_power(layout: Vec<RGB>) -> PowerEstimate {
    let mut pixels: Vec<RGB8> = layout.into_iter().map(|p| RGB8 { r: p.r, g: p.g, b: p.b }).collect();
    matrix_protocol::gamma_correct(&mut pixels);
    let current_ma = matrix_protocol::estimate_current_ma(&pixels);
    //todo: apply brightness limited, then return powerReport and send additional data to the powerEstimate
    PowerEstimate {
        current_ma,
        max_current_ma: matrix_protocol::MAX_CURRENT_MA,
        over_limit: current_ma > matrix_protocol::MAX_CURRENT_MA as f32,
    }
}

#[tauri::command]
async fn set_brightness(state: tauri::State<'_, NetworkState>, value: u8) -> Result<(), String>{
    let command= Command::SetBrightness(value);
    send_command(&state, command).await
}

#[tauri::command]
async fn set_ma(state: tauri::State<'_, NetworkState>, value: u16) -> Result<(), String>{
    let command = Command::SetAmper(value);
    send_command(&state, command).await
}

// async fn get_ma(state: tauri::State<'_, NetworkState>) -> Result<u16, String>{
//
// }

async fn write_command(stream: &mut TcpStream, command: &Command) -> Result<(), String> {
    let mut buf = [0u8; MAX_WIRE_BYTES];
    let len = command.encode(&mut buf[LEN_PREFIX_BYTES..]);

    let len_bytes = (len as u16).to_be_bytes();
    buf[..LEN_PREFIX_BYTES].copy_from_slice(&len_bytes);

    stream
        .write_all(&buf[..LEN_PREFIX_BYTES + len])   // trim to the real message size
        .await
        .map_err(|e| format!("Failed to send command: {}", e))
}

async fn send_command(state: &NetworkState, command: Command) -> Result<(), String> {
    let mut guard = state.0.lock().await;
    let stream = guard.as_mut().ok_or(format!("Not connected"))?;
    write_command(stream, &command).await
}

fn main() {
    tauri::Builder::default()
        .manage(NetworkState(Mutex::new(None)))
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window
                .eval("document.addEventListener('contextmenu', event => event.preventDefault());")
                .unwrap();
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            connect_to_pico,
            disconnect_from_pico,
            send_frame_to_pico,
            estimate_power,
            set_ma,
            set_brightness,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}