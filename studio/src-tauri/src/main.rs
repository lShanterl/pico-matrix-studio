// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;
use std::net::TcpStream;
use std::sync::Mutex;
use serialport::SerialPort;
use matrix_protocol::{Command, Frame, MATRIX_PIXEL_COUNT, MAX_COMMAND_BYTES, PORT};
use tauri::{AppHandle, Emitter, Manager};
use serde::Deserialize;
use bytemuck::{Pod, Zeroable};
use smart_leds::{RGB8};

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
fn send_frame_to_pico(state: tauri::State<NetworkState>, layout: Vec<RGB>) {
    let mut state = state.0.lock().unwrap();

    if let Some(stream) = state.as_mut() {
        let pixels: Vec<RGB8> = layout.into_iter().map(|p| RGB8 { r: p.r, g: p.g, b: p.b }).collect();

        let frame: Frame = match pixels.try_into() {
            Ok(frame) => frame,
            Err(_) => {
                eprintln!("Layout size does not match MATRIX_PIXEL_COUNT");
                return;
            }
        };

        let mut send_buffer = [0u8; MAX_COMMAND_BYTES];
        let command = Command::SetFrame(frame);
        let bytes_written = command.encode(&mut send_buffer);

        let len_prefix = (bytes_written as u16).to_be_bytes();

        if let Err(e) = stream.write_all(&len_prefix)
            .and_then(|_| stream.write_all(&send_buffer[..bytes_written]))
        {
            eprintln!("Failed to send frame: {}", e);
        }

        println!("Sent frame");
    }
    println!("invoked sent frame");
}

#[tauri::command]
fn connect_to_pico(
    app: AppHandle,
    state: tauri::State<NetworkState>,
    ip: String)
{
    println!("Connecting to pico...");
    let mut state = state.0.lock().unwrap();
    let addr = format!("{}:{}",ip, PORT);

    match TcpStream::connect(addr) {
        Ok(stream) => {
            stream.set_nodelay(true).unwrap();

            *state = Some(stream);

            app.emit("pico-connection-status", ConnectionStatus{
                connected: true,
                ip: Some(ip),
                error: None
            }).ok();
        }
        Err(er) => {
            app.emit("pico-connection-status", ConnectionStatus{
                connected: false,
                ip: None,
                error: Some(er.to_string()),
            }).ok();
            println!("Failed to connect to pico: {}", er);
        }
    }
}

#[tauri::command]
fn disconnect_from_pico(app: AppHandle, state: tauri::State<NetworkState>) {
    let mut state = state.0.lock().unwrap();

    if let Some(stream) = state.take() {
        stream.shutdown(std::net::Shutdown::Both).unwrap();

        app.emit("pico-connection-status", ConnectionStatus{
            connected: false,
            ip: None,
            error: None
        }).ok();
    }

    println!("Disconnected from pico");
}


fn main() {
    tauri::Builder::default()
        .manage(NetworkState(Mutex::new(None)))
        .setup(|app| {
            let window = app.get_webview_window("main").unwrap();
            window.eval("document.addEventListener('contextmenu', event => event.preventDefault());").unwrap();
            Ok(())
        })
    .invoke_handler(tauri::generate_handler![
        connect_to_pico, disconnect_from_pico, send_frame_to_pico /*, disconnect, send_frame, set_brightness, select_animation, upload_animation*/
    ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
