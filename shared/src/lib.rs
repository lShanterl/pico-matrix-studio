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
                write_pixels(buf, frame);
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

