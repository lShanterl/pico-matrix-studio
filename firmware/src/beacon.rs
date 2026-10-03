use embassy_net::{IpEndpoint, Ipv4Address, Stack};
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_time::Timer;
use log::info;
use matrix_protocol::{BEACON_MAGIC, BEACON_PORT, PORT};

#[embassy_executor::task]
pub async fn beacon_task(stack: Stack<'static>) {
    // beacon message is 7 bytes: 7 magic vbytes
    let mut rx_meta = [PacketMetadata::EMPTY; 1];
    let mut rx_buf = [0u8; 16];
    let mut tx_meta = [PacketMetadata::EMPTY; 2];
    let mut tx_buf = [0u8; 14]; // 2 messages of 7 bytes, one is enough but keeping 14 for potential info like device name in the future
    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buf, &mut tx_meta, &mut tx_buf);
    socket.bind(0).unwrap();

    let dest = IpEndpoint::new(Ipv4Address::new(255, 255, 255, 255).into(), BEACON_PORT);

    loop{
        stack.wait_config_up().await;
        if let Err(e) = socket.send_to(BEACON_MAGIC, dest).await {
            info!("beacon send failed: {:?}", e);
        }
        Timer::after_secs(2).await;
    }
}
