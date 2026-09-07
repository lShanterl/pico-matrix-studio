use embassy_net::IpListenEndpoint;
use embassy_net::tcp::TcpSocket;
use embassy_net::udp::PacketMetadata;
use log::info;
use matrix_protocol::{Command, MAX_ANIMATION_FRAMES, MAX_COMMAND_BYTES, PORT};
use crate::animations::STORED_ANIMATION;
use crate::COMMAND_CHANNEL;

async fn read_exact(
    socket: &mut TcpSocket<'_>,
    buf: &mut [u8],
) -> Result<(), embassy_net::tcp::Error> {
    let mut read = 0;

    while read < buf.len() {
        let n = socket.read(&mut buf[read..]).await?;
        if n == 0 {
            return Err(embassy_net::tcp::Error::ConnectionReset);
        }
        read += n;
    }

    Ok(())
}

// growing matrix past 16x16 might require to split TCP packets
#[embassy_executor::task]
async fn control_task(stack: embassy_net::Stack<'static>) {
    let mut rx = [0u8; 2048];
    let mut tx = [0u8; 256];
    let mut expected_frames = 0;

    loop{
        let mut socket = TcpSocket::new(
            stack,
            &mut rx,
            &mut tx
        );
        let listen_endpoint = IpListenEndpoint {addr: None, port: PORT};
        if socket.accept(listen_endpoint).await.is_err() {continue};

        info!("Studio app connected");

        loop {

            let mut len_buf = [0u8;2];

            if read_exact(&mut socket, &mut len_buf).await.is_err() { break }

            let len = u16::from_be_bytes(len_buf) as usize;

            let mut payload = [0u8; matrix_protocol::MAX_COMMAND_BYTES];
            if len > payload.len() { break; }

            if read_exact(&mut socket, &mut payload[..len]).await.is_err() { break }

            match Command::decode(&payload[..len]) {
                Ok(Command::UploadAnimationStart { frame_count, fps }) =>{
                    expected_frames = frame_count.min(MAX_ANIMATION_FRAMES as u8);
                    let mut anim = STORED_ANIMATION.lock().await;
                    anim.frame_count = 0; // invalidate till end
                    anim.fps = fps.max(1);
                }
                Ok(Command::UploadAnimationEnd) =>{
                    let mut anim = STORED_ANIMATION.lock().await;
                    anim.frame_count = expected_frames as usize;
                    info!("Animation stored: {} frames @ {} fps", anim.frame_count, anim.fps);
                }
                Ok(Command::UploadAnimationFrame {index, frame}) => {
                    let mut anim = STORED_ANIMATION.lock().await;
                    anim.frames[index as usize] = frame;
                }
                Ok(cmd) => {COMMAND_CHANNEL.send(cmd).await} // SetFrame, SetBrightness, SelectAnimation, PlayUploadedAnimation
                Err(e) => info!("Bad command: {:?}", e),
            }

        }
        info!("Studio app disconnected, waiting for connection");


    }

}
