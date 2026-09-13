// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bytemuck::{Pod, Zeroable};
use matrix_protocol::{Command, Frame, MATRIX_PIXEL_COUNT, MAX_COMMAND_BYTES, PORT};
use serde::Deserialize;
use serialport::SerialPort;
use smart_leds::{RGB8};
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

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[tauri::command]
async fn send_frame_to_pico(state: tauri::State<'_,NetworkState>, layout: Vec<RGB>) -> Result<(), String> {
    let mut guard = state.0.lock().await;

    if let Some(stream) = guard.as_mut() {
        // convert to smart-leds RGB8
        let pixels = layout.iter().map(|p| RGB8{ r: p.r, g: p.g, b: p.b }).collect::<Vec<_>>();

        let frame: Frame = match pixels.try_into() {
            Ok(frame) => frame,
            Err(_) => {
                eprintln!("Layout size does not match MATRIX_PIXEL_COUNT");
                return Result::Err(String::from("Layout size does not match MATRIX_PIXEL_COUNT"));
            }
        };

        let mut send_buffer = [0u8; MAX_COMMAND_BYTES];
        let command = Command::SetFrame(frame);
        let bytes_written = command.encode(&mut send_buffer);

        let len_prefix = (bytes_written as u16).to_be_bytes();

        if let Err(e) = async move {
            stream.write_all(&len_prefix).await?;
            stream.write_all(&send_buffer[..bytes_written]).await?;
            Ok::<(), std::io::Error>(())
        }.await {
            eprintln!("Failed to send frame: {}", e);
            return Err(format!("Failed to send frame: {}", e));
        }

        println!("Sent frame");
    }
    return Result::Ok(());
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
            send_frame_to_pico
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}