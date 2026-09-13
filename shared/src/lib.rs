#![no_std]
pub const MATRIX_WIDTH: usize = 16;
pub const MATRIX_HEIGHT: usize = 16;
pub const MATRIX_PIXEL_COUNT: usize = MATRIX_WIDTH * MATRIX_HEIGHT;
pub static MAX_CURRENT_MA: u32 = 1500;
pub const MAX_ANIMATION_FRAMES: usize = 30;
pub const FRAME_BYTES: usize = MATRIX_PIXEL_COUNT * 3; // each color

// Largest single command on the wire: tag + 2-byte index + one full frame
pub const MAX_COMMAND_BYTES: usize = 1 + 2 + FRAME_BYTES;
pub const PORT: u16 = 7778;

use smart_leds::RGB8;
pub type Frame = [RGB8; MATRIX_PIXEL_COUNT];

// thanks to https://victornpb.github.io/gamma-table-generator/
pub const GAMMA8: [u8; 256] = [
    0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,   0,
    0,   0,   0,   0,   0,   1,   1,   1,   1,   1,   1,   1,   1,   1,   1,   1,
    1,   1,   1,   1,   2,   2,   2,   2,   2,   2,   2,   2,   2,   3,   3,   3,
    3,   3,   3,   4,   4,   4,   4,   4,   4,   5,   5,   5,   5,   6,   6,   6,
    6,   6,   7,   7,   7,   8,   8,   8,   8,   9,   9,   9,  10,  10,  10,  11,
    11,  12,  12,  12,  13,  13,  14,  14,  14,  15,  15,  16,  16,  17,  17,  18,
    18,  19,  19,  20,  21,  21,  22,  22,  23,  23,  24,  25,  25,  26,  27,  27,
    28,  29,  30,  30,  31,  32,  33,  33,  34,  35,  36,  37,  37,  38,  39,  40,
    41,  42,  43,  44,  45,  46,  47,  48,  49,  50,  51,  52,  53,  54,  55,  56,
    57,  59,  60,  61,  62,  63,  65,  66,  67,  68,  70,  71,  72,  74,  75,  76,
    78,  79,  81,  82,  84,  85,  87,  88,  90,  91,  93,  95,  96,  98,  99, 101,
    103, 105, 106, 108, 110, 112, 113, 115, 117, 119, 121, 123, 125, 127, 129, 131,
    133, 135, 137, 139, 141, 143, 146, 148, 150, 152, 154, 157, 159, 161, 164, 166,
    168, 171, 173, 176, 178, 181, 183, 186, 188, 191, 194, 196, 199, 202, 204, 207,
    210, 213, 216, 219, 221, 224, 227, 230, 233, 236, 239, 242, 246, 249, 252, 255,
];

pub fn gamma_correct(pixels: &mut [RGB8]) {
    for p in pixels.iter_mut() {
        p.r = GAMMA8[p.r as usize];
        p.g = GAMMA8[p.g as usize];
        p.b = GAMMA8[p.b as usize];
    }
}

// Estimates the current consumption of the LED matrix in milliamperes based on the pixel colors. Used as precaution in order not to burn the charger
pub fn estimate_current_ma(pixels: &[RGB8]) -> f32 {
    let idle = pixels.len() as f32;
    let mut sum: u32 = pixels.iter().map(|p| p.r as u32 + p.g as u32 + p.b as u32).sum();

    (sum as f32 * 20.0 / 255.0) + idle
}


#[derive(Debug)]
pub enum Command {
    SetFrame(Frame),
    SetBrightness(u8),
    SelectAnimation(u8),
    UploadAnimationStart { frame_count: u8, fps: u8 },
    UploadAnimationFrame { index: u8, frame: Frame },
    UploadAnimationEnd,
    PlayUploadedAnimation,
}

#[derive(Debug)]
pub enum DecodeError {
    UnknownTag(u8),
    TooShort,
}

fn write_pixels(buf: &mut[u8], pixels: &Frame) {
    for(i, p) in pixels.iter().enumerate() {
        buf[i * 3] = p.r;
        buf[i * 3 + 1] = p.g;
        buf[i * 3 + 2] = p.b;
    }
}
fn read_pixels(buf: &[u8]) -> Frame {
    let mut pixels = [RGB8::default(); MATRIX_PIXEL_COUNT];

    for(i, p) in pixels.iter_mut().enumerate() {
        p.r = buf[i * 3];
        p.g = buf[i * 3 + 1];
        p.b = buf[i * 3 + 2];
    }
    pixels
}


impl Command {
    pub fn encode(&self, buf: &mut [u8]) -> usize {
        match self {
            Command::SetFrame(frame) => {
                buf[0] = 0x01;
                //todo check whether im doing this right
                write_pixels(&mut buf[1..], frame);
                1 + FRAME_BYTES
            }
            Command::SetBrightness(brightness) => {
                buf[0] = 0x02;
                buf[1] = *brightness;
                2
            }
            Command::SelectAnimation(animation) => {
                buf[0] = 0x03;
                buf[1] = *animation;
                2
            }
            Command::UploadAnimationStart { frame_count, fps } => {
                buf[0] = 0x04;
                buf[1] = *frame_count;
                buf[2] = *fps;
                3
            }
            Command::UploadAnimationFrame { index, frame } => {
                buf[0] = 0x05;
                buf[1] = *index;

                write_pixels(&mut buf[2..], frame);
                2 + FRAME_BYTES
            }
            Command::UploadAnimationEnd => {
                buf[0] = 0x06;
                1
            }
            Command::PlayUploadedAnimation => {
                buf[0] = 0x07;
                1
            }

        }
    }

    pub fn decode(buf: &[u8]) -> Result<Command, DecodeError> {
        let tag = buf[0];

        match tag {
            0x01 =>{
                if buf.len() < 1 + FRAME_BYTES { return Err(DecodeError::TooShort); }
                Ok(Command::SetFrame(read_pixels(&buf[1..])))
            }
            0x02 =>{
                if buf.len() < 2 { return Err(DecodeError::TooShort); }
                Ok(Command::SetBrightness(buf[1]))
            }
            0x03 => {
                if buf.len() < 2 { return Err(DecodeError::TooShort); }
                Ok(Command::SelectAnimation(buf[1]))
            }
            0x04 => {
                if buf.len() < 3 { return Err(DecodeError::TooShort); }
                Ok(Command::UploadAnimationStart { frame_count:  buf[1], fps: buf[2] })
            }
            0x05 =>{
                if buf.len() < 2 + FRAME_BYTES { return Err(DecodeError::TooShort); }
                Ok(Command::UploadAnimationFrame { index: buf[1], frame: read_pixels(&buf[2..]) })
            }
            0x06 =>{
                Ok(Command::UploadAnimationEnd)
            }
            0x07 =>{
                Ok(Command::PlayUploadedAnimation)
            }
            _ => Err(DecodeError::UnknownTag(tag)),
        }
    }
}

