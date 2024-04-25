use std::net::{Ipv4Addr, UdpSocket};
use std::time::Duration;

use anyhow::{anyhow as e, Result};

pub fn connect(address: &str) -> Result<UdpSocket> {
    let from_address = (Ipv4Addr::UNSPECIFIED, 0);
    let socket = UdpSocket::bind(from_address).map_err(|e| e!("udp::connect: {}", e))?;
    socket.connect(address)?;
    Ok(socket)
}

pub fn send(address: &str, message: &[u8]) -> Result<UdpSocket> {
    let socket = connect(address)?;
    socket.send(message).map_err(|e| e!("udp::send: {}", e))?;
    Ok(socket)
}

pub fn receive(socket: &UdpSocket, timeout: Option<Duration>) -> Result<Vec<u8>> {
    let mut buffer = [0; 8 * 1024];
    socket.set_read_timeout(timeout)?;
    let bytes_read = socket
        .recv(&mut buffer)
        .map_err(|e| e!("udp::receive: {}", e))?;
    let response = &buffer[..bytes_read];
    Ok(Vec::from(response))
}

pub fn send_and_receive(
    address: &str,
    message: &[u8],
    timeout: Option<Duration>,
) -> Result<Vec<u8>> {
    let socket = send(address, message)?;
    receive(&socket, timeout)
}
