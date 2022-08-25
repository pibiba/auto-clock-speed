use super::daemon::Daemon;
use super::logger;
use super::logger::Interface;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::BufWriter;
use std::num::ParseIntError;
use std::os::unix::net::UnixListener;
use std::sync::Arc;
use std::sync::Mutex;

pub mod hook;
pub mod listen;

#[derive(PartialEq, Eq, Debug)]
pub enum Packet {
    Hello(String),
    HelloResponse(String, u32),
    Unknown,
}

#[derive(Debug)]
pub struct PacketParseError;

impl Display for PacketParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Packet parse error occured")
    }
}

impl From<ParseIntError> for PacketParseError {
    fn from(_err: ParseIntError) -> Self {
        PacketParseError
    }
}

pub fn parse_packet(packet: &str) -> Result<Packet, PacketParseError> {
    let mut packet_split = packet.split('|');
    let packet_type = packet_split.next().ok_or(PacketParseError)?;
    let packet_data = packet_split.next().ok_or(PacketParseError)?;
    match packet_type {
        "0" => Ok(Packet::Hello(packet_data.to_string())),
        "1" => Ok(Packet::HelloResponse(
            packet_data.to_string(),
            packet_split.next().unwrap().parse::<u32>()?,
        )),
        _ => Err(PacketParseError),
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Packet::Hello(data) => writeln!(f, "0|{}", data),
            Packet::HelloResponse(data, version) => writeln!(f, "1|{}|{}", data, version),
            Packet::Unknown => writeln!(f),
        }
    }
}

fn log_to_daemon(daemon: &Arc<Mutex<Daemon>>, message: &str, severity: logger::Severity) {
    let mut daemon = daemon.lock().unwrap();
    daemon.logger.log(message, severity);
}

#[cfg(test)]
mod tests {
    use crate::network::{parse_packet, Packet};

    #[test]
    fn parse_packet_test() {
        assert!(parse_packet("0|test").unwrap() == Packet::Hello("test".to_string()));
        assert!(parse_packet("1|test|5").unwrap() == Packet::HelloResponse("test".to_string(), 5));
        assert!(parse_packet("0|test").unwrap() != Packet::HelloResponse("test".to_string(), 5));
    }
}