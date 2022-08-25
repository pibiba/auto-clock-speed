use crate::logger;
use crate::network::log_to_daemon;
use crate::network::parse_packet;
use crate::network::BufWriter;
use crate::network::Daemon;
use crate::network::Packet;
use crate::network::UnixListener;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

pub fn listen(path: &'static str, c_daemon_mutex: Arc<Mutex<Daemon>>) {
    thread::spawn(move || {
        // Get rid of the old sock
        std::fs::remove_file(path).ok();

        // Try to handle sock connections then
        let listener = match UnixListener::bind(path) {
            Ok(listener) => listener,
            Err(e) => {
                log_to_daemon(
                    &c_daemon_mutex,
                    &format!("Failed to bind to {}: {}", path, e),
                    logger::Severity::Error,
                );
                return;
            }
        };

        // Set the permissions on the sock
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o777)).ok();

        // Spawn a new thread to listen for commands
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        handle_stream(stream, &c_daemon_mutex);
                    }
                    Err(err) => {
                        log_to_daemon(
                            &c_daemon_mutex,
                            &format!("Failed to accept connection: {}", err),
                            logger::Severity::Error,
                        );
                        break;
                    }
                }
            }
        });
    });
}

pub fn handle_stream(stream: UnixStream, c_daemon_mutex: &Arc<Mutex<Daemon>>) {
    log_to_daemon(c_daemon_mutex, "Received connection", logger::Severity::Log);

    let inner_daemon_mutex = c_daemon_mutex.clone();

    thread::spawn(move || {
        let reader = BufReader::new(&stream);
        for line in reader.lines() {
            let actual_line = match line {
                Ok(line) => line,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => {
                        return;
                    }
                    _ => {
                        log_to_daemon(
                            &inner_daemon_mutex.clone(),
                            &format!("Failed to read line: {}", e),
                            logger::Severity::Error,
                        );
                        return;
                    }
                },
            };
            let packet = match parse_packet(&actual_line) {
                Ok(p) => p,
                Err(e) => {
                    log_to_daemon(
                        &inner_daemon_mutex.clone(),
                        &format!("Received malfomed packet: {}", e),
                        logger::Severity::Error,
                    );
                    Packet::Unknown
                }
            };
            match packet {
                Packet::Hello(hi) => {
                    let hello_packet = Packet::HelloResponse(hi.clone(), 0);
                    log_to_daemon(
                        &inner_daemon_mutex.clone(),
                        &format!("Received hello packet: {}", hi),
                        logger::Severity::Log,
                    );
                    let mut writer = BufWriter::new(&stream);
                    writer
                        .write_all(format!("{}", hello_packet).as_bytes())
                        .unwrap();
                    writer.flush().unwrap();
                }
                Packet::HelloResponse(_, _) => {}
                Packet::Unknown => {}
            };
        }
    });
}