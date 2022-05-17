use std::fmt::{Display, Formatter};
use std::sync::{Arc, Mutex};
use std::io::{BufWriter, Read, Write};
use std::net::{Shutdown, TcpStream};
use std::ops::{Deref, DerefMut};
use std::collections::HashMap;
use std::rc::Rc;
use std::thread;
use crate::client::client_message_utils::{build_client_headers, build_connect_request, build_create_lobby_request, build_join_lobby_request, build_start_game_request, parse_connect_response, parse_lobby_info_response, parse_lobby_list_response, parse_server_message_header, parse_supported_games_response};
use crate::enums::{MessageType, ProtocolState, StatusCode};
use crate::game_module::{GameMetadata, GameModule};
use crate::shared_data::Lobby;

mod client_message_utils;

pub struct GameProtocolClient {
    ip: Option<String>,
    port: Option<String>,
    client_id: String,
    socket: Option<TcpStream>,
    protocol_state: ProtocolState,
    current_lobby: Option<Lobby>,
    lobbies: Vec<Lobby>,
    supported_games: HashMap<String, Rc<dyn GameModule>>,
    matching_supported_game_descriptions: Vec<GameMetadata>,
    next_message_num: u32
}

impl GameProtocolClient {
    pub fn new() -> Self {
        Self {
            protocol_state: ProtocolState::Closed,
            current_lobby: None,
            lobbies: vec![],
            supported_games: HashMap::new(),
            matching_supported_game_descriptions: vec![],
            socket: None,
            next_message_num: 0,
            ip: None,
            port: None,
            client_id: String::from("")
        }
    }

    pub fn get_socket_address(&self) -> String {
        if let (Some(ip), Some(port)) = (&self.ip, &self.port) {
            return format!("{}:{}", ip, port);
        }
        String::from("")
    }

    pub fn get_protocol_state(&self) -> ProtocolState {
        self.protocol_state
    }

    pub fn get_supported_games(&self) -> Vec<GameMetadata> {
        self.matching_supported_game_descriptions.clone()
    }

    pub fn get_lobby_list(&self) -> Vec<Lobby> {
        self.lobbies.clone()
    }

    pub fn get_current_lobby(&self) -> Option<Lobby> {
        self.current_lobby.clone()
    }

    pub fn get_client_id(&self) -> String {
        self.client_id.clone()
    }

    pub fn register_game<T: 'static + GameModule>(&mut self) {
        let new_game = Rc::new(T::new());
        let metadata = GameMetadata::from(new_game.clone());
        self.supported_games.insert(metadata.to_string(), new_game);
    }

    pub fn request_lobby_list(&mut self) {
        self.protocol_state = ProtocolState::GettingLobbies;
        self.send_message(build_client_headers(self.next_message_num, MessageType::LobbyListRequest));
        self.listen();
    }

    pub fn request_supported_games(&mut self) {
        self.protocol_state = ProtocolState::GettingSupportedGames;
        self.send_message(build_client_headers(self.next_message_num, MessageType::SupportedGamesRequest));
        self.listen();
    }

    pub fn refresh_current_lobby(&mut self) {
        self.protocol_state = ProtocolState::GettingLobbyInfo;
        self.send_message(build_client_headers(self.next_message_num, MessageType::LobbyInfoRequest));
        self.listen();
    }

    pub fn create_lobby(&mut self, game_key: &str) {
        self.protocol_state = ProtocolState::CreatingLobby;
        self.send_message(build_create_lobby_request(self.next_message_num, game_key.to_string()));
        self.listen();
    }

    pub fn join_lobby(&mut self, lobby_id: &str) {
        self.protocol_state = ProtocolState::JoiningLobby;
        self.send_message(build_join_lobby_request(self.next_message_num, lobby_id.to_string()));
        self.listen();
    }

    pub fn leave_lobby(&mut self) {
        self.protocol_state = ProtocolState::LeavingLobby;
        self.send_message(build_client_headers(self.next_message_num, MessageType::LeaveLobbyRequest));
        self.listen();
    }

    pub fn start_game(&mut self, lobby_id: String) {
        self.protocol_state = ProtocolState::CreatingGameSession;
        self.send_message(build_start_game_request(self.next_message_num, lobby_id));
        self.listen();
    }

    pub fn connect(&mut self, ip: &String, port: &String) {
        // Return if socket is Some (has a value, already connected).
        if self.socket.is_some() {
            return;
        }

        // Set the protocol state and save the ip and port
        self.protocol_state = ProtocolState::Authenticating;
        self.ip = Some(ip.clone());
        self.port = Some(port.clone());

        // If it is None (unassigned, not connected), then try to connect.
        match TcpStream::connect(self.get_socket_address()) {
            Ok(tcp_stream) => {

                // Store the socket and send a connect request.
                self.socket = Some(tcp_stream);

                self.send_message(build_connect_request(self.next_message_num, None));
                self.listen();
            }
            Err(e) => {
                self.protocol_state = ProtocolState::Closed;
                println!("Connect error: {}", e);
            }
        }
    }

    pub fn disconnect(&mut self) {
        self.protocol_state = ProtocolState::ClosingConnection;
        self.send_message(build_client_headers(self.next_message_num, MessageType::DisconnectRequest));
        self.listen();
    }

    fn send_message(&mut self, data: Vec<u8>) {
        if let Some(socket) = &mut self.socket {
            if self.next_message_num == u32::MAX {
                self.next_message_num = 0;
            } else {
                self.next_message_num += 1;
            }
            socket.write(data.as_slice());
        }
    }

    fn listen(&mut self) {
        if let Some(socket) = &mut self.socket {
            // Read stream data into the buffer.
            let mut buffer = [0; 1024]; // 1024 byte buffer
            match socket.read(&mut buffer) { // TODO should I set timeout?
                Ok(size) => {
                    // If size is more than 0, then this is a legit message we are receiving.
                    // If size is 0, then socket is closed, so formally shut it down.
                    if size > 0 {
                        let (status_code, message_type, remainder) = parse_server_message_header(&buffer);
                        println!("From server: {:?}, {:?}, {}, {}", status_code, message_type, remainder.len(), size);

                        match message_type {
                            MessageType::ConnectResponse => {
                                // Only accept the ConnectResponse if it was successful and this client_bin was in the correct state: Authenticating.
                                if matches!(status_code, StatusCode::Success) {
                                    let response = parse_connect_response(remainder);
                                    self.client_id = response.client_id;
                                    self.protocol_state = ProtocolState::Idle;
                                }
                            }
                            MessageType::DisconnectResponse => {
                                if matches!(status_code, StatusCode::Success) {
                                    self.protocol_state = ProtocolState::Closed;
                                    self.current_lobby = None;
                                    self.client_id = String::from("");
                                    self.lobbies = vec![];
                                    self.matching_supported_game_descriptions = vec![];
                                    self.next_message_num = 0;
                                    socket.shutdown(Shutdown::Both).unwrap();
                                }
                            }
                            MessageType::LobbyListResponse => {
                                if matches!(status_code, StatusCode::Success) {
                                    let data = parse_lobby_list_response(remainder);
                                    self.protocol_state = ProtocolState::Idle;
                                    self.lobbies = data.lobbies;
                                }
                            }
                            MessageType::SupportedGamesResponse => {
                                if matches!(status_code, StatusCode::Success) {
                                    let data = parse_supported_games_response(remainder);

                                    // Compare list of server_bin supported games with client_bin supported games.
                                    // Collect the matching games and store them since these are the ones the client_bin should only be able to create lobbies for and join.
                                    let mut matching_games = vec![];
                                    for server_game in data.games.iter() {
                                        let metadata_key = server_game.to_string();
                                        if self.supported_games.contains_key(&metadata_key) {
                                            matching_games.push(GameMetadata::from(self.supported_games.get(&metadata_key).unwrap().clone()))
                                        }
                                    }
                                    self.protocol_state = ProtocolState::Idle;
                                    self.matching_supported_game_descriptions = matching_games;
                                }
                            }
                            MessageType::LobbyInfoResponse => {
                                // TODO should we handle joining lobby and just getting the info separately?
                                if matches!(status_code, StatusCode::Success) {
                                    let data = parse_lobby_info_response(remainder);
                                    self.protocol_state = ProtocolState::InLobby;
                                    self.current_lobby = Some(data.lobby);
                                }
                            }
                            MessageType::LeaveLobbyResponse => {
                                if matches!(status_code, StatusCode::Success) {
                                    self.protocol_state = ProtocolState::Idle;
                                    self.current_lobby = None;
                                }
                            }
                            MessageType::ProtocolError => {
                                // TODO handle all kinds of errors. Based on the previously set protocol state.
                                //  For example, if we are creating a lobby and receive a protocol error, then it's going to
                                //  be an error about creating the lobby. So handle it appropriately.
                            }
                            MessageType::GameStateResponse => {}
                            MessageType::ResendMessageRequest => {}
                            MessageType::UnsolicitedMessage => {}
                            MessageType::Unsupported => {}
                            _ => {} // Default and Unsupported. Do nothing if we get a message unsupported on the client_bin side.
                        };
                    } else {
                        socket.shutdown(Shutdown::Both);
                    }
                }
                Err(e) => {
                    println!("Listen error: {}", e);
                }
            };
        }
    }
}
