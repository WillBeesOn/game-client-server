use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::ops::{Deref, DerefMut};
use std::thread;
use crate::protocol::protocol::ConnectionStatus::Connected;

#[derive(Debug, Clone, Copy)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected
}

pub struct GameProtocolState {
    connection_status: ConnectionStatus,
    ip: String,
    port: String
}

pub struct StreamReader {
    state: Arc<Mutex<GameProtocolState>>,
    socket: Option<TcpStream>
}

pub struct StreamWriter {
    state: Arc<Mutex<GameProtocolState>>,
    socket: Option<TcpStream>
}

pub struct GameProtocolClient {
    state: Arc<Mutex<GameProtocolState>>,
    reader: Arc<StreamReader>,
    writer: Arc<StreamWriter>
}

impl GameProtocolClient {
    pub fn new(ip: &str, port: &str) -> Self {
        let state = GameProtocolState {
            connection_status: ConnectionStatus::Disconnected,
            ip: ip.to_string(),
            port: port.to_string()
        };

        // Creat a mutex arc so state can be handled in reading, writing, and main (gui) threads.
        // Be sure to unlock it with drop(guard) when done accessing it.
        let state_arc = Arc::new(Mutex::new(state));

        let reader = StreamReader {
            state: state_arc,
            socket: None
        };

        let writer = StreamWriter {
            state: reader.state.clone(),
            socket: None
        };
        Self {
            state: reader.state.clone(),
            reader: Arc::new(reader),
            writer: Arc::new(writer)
        }
    }

    pub fn get_connection_status(&self) -> ConnectionStatus {
        self.state.lock().unwrap().connection_status
    }

    pub fn connect(&mut self) {
        // Return if socket is Some (has a value, already connected).
        let mut state_guard = self.state.lock().unwrap();
        if !matches!(state_guard.connection_status, ConnectionStatus::Disconnected) {
            return;
        }

        // If it is None (unassigned, not connected), then try to connect.
        match TcpStream::connect(format!("{}:{}", state_guard.ip, state_guard.port)) {
            Ok(tcp_stream) => {
                state_guard.connection_status = Connected;
                drop(state_guard);
                Arc::get_mut(&mut self.reader).unwrap().socket = Some(tcp_stream.try_clone().unwrap());
                Arc::get_mut(&mut self.writer).unwrap().socket = Some(tcp_stream.try_clone().unwrap());
                self.listen();
            },
            Err(e) => {
                println!("{}", e);
            }
        }
    }

    pub fn send_message(&mut self) {
        let mut state_guard = self.state.lock().unwrap();
        if !matches!(state_guard.connection_status, ConnectionStatus::Connected) {
            return;
        }
        drop(state_guard);

        let writer_clone = self.writer.clone();
        thread::spawn(move|| {
            writer_clone.socket.as_ref().unwrap().write(b"test").unwrap();
        });
    }

    pub fn listen(&self) {
        let mut state_guard = self.state.lock().unwrap();
        if !matches!(state_guard.connection_status, ConnectionStatus::Connected) {
            return;
        }
        drop(state_guard);

        let reader_clone = self.reader.clone();
        thread::spawn(move|| {
            loop {
                let mut buffer = [0; 1024]; // 1024 byte buffer

                // Read stream data into the buffer.
                reader_clone.socket.as_ref().unwrap().read(&mut buffer).unwrap();

                println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
            }
        });
    }
}

fn write(stream: &mut TcpStream, message: &str) {
    stream.write(message.as_bytes()).unwrap();
}
