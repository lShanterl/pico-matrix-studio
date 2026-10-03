# Pico LED Matrix Studio
A Wi-Fi enabled, custom-built smart LED matrix powered by a Raspberry Pi Pico W and WS2812 LEDs, driven by a highly optimized asynchronous Rust firmware and controlled via a feature-rich Tauri desktop companion application.

## Table of Contents
- [Overview](#overview)
- [Key Features](#key-features)
- [Software Design](#software-design)
- [Hardware Design](#hardware-design)
- [Bill of Materials](#bill-of-materials)
- [Build & Running](#build-&-running)
- [Studio Workflow](#studio-workflow)

## Overview
The Pico LED Matrix Studio is a complete hardware and software ecosystem for creating and displaying pixel art animations over a local network. The project encompasses a custom physical build utilizing a Pico W, a robust embedded Rust firmware handling network communications and display rendering, and a powerful Tauri-based frontend (React/TypeScript + Rust backend) for drawing and matrix management.

## Key Features
- **UDP Auto-Discovery Beacon:** The Pico W continously broadcasts a UDP magic packet to `255.255.255.255`. The Tauri studio app listens for this beacon to dynamically find the device and estabilish a TCP connection without requiring any manual IP configuration.
- **Intelligent Power Management & Safety:** Features dynamic brightness adjustments alongside a strict maximum amperage limit to protect USB ports and power supplies from overcurrent draw. The Tauri app calculates live power estimations before sending frames, and the firmware actively limits brightness if the layout exceeds the configured safe threshold. Setting new power limits requires a 3 second "hold-to-confirm" action in the app to prevent accidental power spikes.
- **Efficient Flash Storage:** Uploaded animations and device settings are efficiently saved to Pico's flash memory to minimize its wear over time. The storage system utilizes page-aligned writes, versioning, and CRC32 checksums to prevent memory corruption if the device is unexpectedly unplugged during a save operation.
- **Advanced Drawing Studio:** A fully featured workspace including a Pencil, Eraser, color Pipette, and a Bucket fill tool for rapid drawing. It supports simple Undo/Redo functionality by maintaining operation history.
- **Animation & FPS Engine:** Support for creating multi-frame animations, each with a distinct Frames Per Second (FPS) configuration. If no custom animation is playing the firmware falls back to optimized 60FPS procedural animations (like a rotating plasma effect).
- **Hardware-Agnostic Drawing (Matrix Transform):** if your matrix is mounted upside down or sideways, you can apply software-level rotations (0°, 90°, 180°, 270°) and horizontal/vertical mirroring in the settings, so your drawing map perfectly to the hardware.
- **Custom Palettes & Color:** Pick precise colors using HEX codes or a native color picker, and save them to a quick-access palette.
- **DTR Debug Mode:** For development, the firmware includes a USB CDC logger. When compiled with the debug feature, the boot sequence pauses and waits until a serial terminal (such as PuTTY) connects via a DTR signal before proceeding and printing logs.

## Software Design
### The Firmware (Embedded Rust)
The firmware is written entirely in embedded Rust utilizing the **Embassy** framework for efficient, asynchronous task management. The system concurrently runs isolated task for Wi-Fi management, a TCP command listener, a UDP discovery beacon and background flash storage handling.

To ensure maximum performance, the WS2812 LEDs are driven using the RP2040's PIO (Programmable I/O) state machines combined with DMA (Direct Memory Access) channels. This offloads the strict timing requirements of the WS2812 protocol from the main CPU.

#### Concurrency & Inter-Task Communication
- **Channel:** A ``Channel<CriticalSectionRawMutex,4 Command>`` acts as the bridge betweem the network and the display. When the TCP task receives and decodes a network packet, it sends the parsed ``Command`` through this channel to the main rendering loop. This ensures the network listener is never blocked, while the main loop applies the commands perfectly in sync with matrix refresh rate.
- **Mutexes:** Global device state (brightness, maximum amperage limit, and current animation frames) is protected by a ``Mutex<CriticalSectionRawMutex, DeviceStorage>``. This critical-section mutex safely shares the device state across the TCP listener, the main rendering loop, and the storage task without causing data races or blocking hardware interrupts.
- **Signals:** To minimize CPU overhead and protect flash memory from unnecessary wear, the background flash storage task sleeps by default. When the network task receives new settings or an animation frame - it triggers a ``Signal<CriticalSectionRawMutex, ()>``. This instantly wakes up the storage task, which checks for "dirty" flags, commits the changes to flash memory and goes back to sleep.

#### The Studio (Tauri + React)
The desktop frontend is built with Tauri, combining a React/TypeScript UI with a Rust backend. The workspace automatically caches your drawings and animation frames to ``localStorage``. The backend Rust process handles TCP socket connections to the Pico, serializing layout data into an optimized binary protocol before transmitting.

## Hardware Design
The physical build is designed for clean light diffusion, easy assembly, and stable power delivery:
- **Diffuser:** A milky plexiglass front plate is used to diffuse harsh WS2812 LEDs, blending the lights into vibrant eye-strain-less colors.
- **Pixel Grid:** A physical grid sits directly over the LED matrix. This isolates each pixel into its own physical cell, preventing most of the light from bleeding into neighboring pixels so that every "pixel" remains sharp, distinct square. The grid also features recesses for the capacitors.
- **Data Protection:** A 330Ω inline resistor is placed on the data line between the Pico's GPIO pin and the WS2812 Data-In pin to prevent impedance issues and protect the microcontroller.
- **Assembly:** Wiring is routed through a quick connector for easy assembly and maintenance without excessive soldering.
- **Power Stability:** A 10µF capacitor is placed across the 5V and GND power lines to smooth out power delivery and prevent sudden inrush currents from damaging the LEDs on startup.
- **Easy BOOTSEL Access:** Features a hole that allows resetting the microcontroller without the need to dismount the entire matrix.

## Bill of Materials

Component | Notes
--------- | --------
Raspberry Pi Pico W | Main microcontroller
WS2812 LED Matrix | Primary display panel
Pixel Separator Grid | Physical baffle to isolate LEDs and prevent light bleed
Milky Plexiglass | Mounted over the grid for pixel light diffusion
10µF Capacitor | Smooths power delivery and prevents LED burn-out
330Ω Resistor | Placed between Pico GPIO and WS2812 data-in pin
Quick Connector | Terminal block for easy component wiring
Power Supply | 5V external power supply (ensure adequate amperage)

## Building & Running
### 1. Building the Firmware (Pico W)
Prerequisities:
- Rust toolchain with the thumbv6m-none-eabi target installed.

#### Network Configuration:
Before compiling, you must set your local Wi-Fi credentials as environment variables so the Pico W can connect to your network upon booting.
```bash
export WIFI_SSID="your_network_name"
export WIFI_PASSWORD="your_network_password"
```
#### Build Commands:
Navigate to firmware directory and build in release mode:
```bash
cargo build --release
```
or
```bash
cargo build --no-default-features
```
### 2. Running the Studio App (Tauri)
Prerequisites:
- Node.js and npm
- Rust toolchain

#### Build Commands:
Navigate to the frontend directory:

```bash
# Install frontend dependencies
npm install

# Run the app in the development mode
npm run tauri dev

# Or build the executable for your OS
npm run tauri build
```
## Studio Workflow
1. **Connect:** Ensure the Pico is powered on. Open the Studio app; click the top left dot, the app will automatically listen for the UDP beacon and populate the IP and then connect.
2. **Configure:** Open the Settings gear. Define your power supply's maximum amperage limit and orient your matrix rotation if it's mounted sideways.
3. **Animate:** Click the ```+``` in the sidebar to create a new animation. Use tools to draw your frames.
4. **Upload:** Adjust the FPS slider to your desired speed, then click the **Play** button on the floating toolbar. The app will bundle the frames and stream them to the Pico saving them to flash memory.
