// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;
use std::net::TcpStream;
use std::sync::Mutex;
use serialport::SerialPort;
use matrix_protocol::{Command, MATRIX_PIXEL_COUNT, MAX_COMMAND_BYTES, PORT};

struct NetState(Mutex<Option<TcpStream>>);

fn send_command(cmd: &Command, state: &NetState) -> Result<(), String> {
    let mut buf = [0u8; MAX_COMMAND_BYTES];
    let len = cmd.encode(&mut buf);

    let mut guard = state.0.lock().unwrap();
    let stream = guard.as_mut().ok_or("not connected")?;
    stream.write_all(&(len as u16).to_be_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&buf[..len]).map_err(|e| e.to_string())
}

#[tauri::command]
fn connect(ip: String, state: tauri::State<NetState>) -> Result<(), String>{
    let stream = TcpStream::connect(format!("{ip}:{PORT}")).map_err(|e| e.to_string())?;
    stream.set_nodelay(true).map_err(|e| e.to_string())?;
    *state.0.lock().unwrap() = Some(stream);
    Ok(())
}

#[tauri::command]
fn ping(str: &str) -> String {
    println!("got {}", str);
    "pong".into()
}


fn main() {
    tauri::Builder::default()
        .manage(NetState(Mutex::new(None)))
    .invoke_handler(tauri::generate_handler![
        connect, ping /*, disconnect, send_frame, set_brightness, select_animation, upload_animation*/
    ])
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
