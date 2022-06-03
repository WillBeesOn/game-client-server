use std::io::{Read, Write};
use std::net::{Shutdown, TcpStream};
use std::collections::HashMap;
use std::os::macos::raw::stat;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use crate::client::client_message_utils::{build_client_headers, build_connect_request, build_create_lobby_request, build_join_lobby_request, build_move_request, build_start_game_request, parse_connect_response, parse_game_state_response, parse_lobby_info_response, parse_lobby_list_response, parse_server_message_header, parse_supported_games_response};
use crate::enums::{MessageType, ProtocolState, StatusCode};
use crate::game_module::{GameModule, GameMove, GameState};
use crate::shared_data::Lobby;

mod client_message_utils;

struct GameProtocolClientState {
    protocol_state: ProtocolState,
    previous_protocol_state: ProtocolState,
    next_message_num: u32,
    client_id: String,
    socket: Option<Arc<TcpStream>>,
    current_lobby: Option<Lobby>,
    lobbies: Vec<Lobby>,
    is_listening_async: bool,
    game_in_progress: Option<Box<dyn GameModule>>,
    supported_games: HashMap<String, Arc<dyn GameModule>>,
    matching_supported_games: Vec<(String, String)>, // Represent the names and IDs of supported games as (GameTitle, GameID)
    on_message_received: Option<Box<dyn Fn() + Send + Sync>>,
}

pub struct GameProtocolClient {
    ip: Option<String>,
    port: Option<String>,
    state: Arc<Mutex<GameProtocolClientState>>,
}

impl GameProtocolClient {
    pub fn new() -> Self {
        let state = Arc::new(Mutex::new(GameProtocolClientState {
            protocol_state: ProtocolState::Closed,
            previous_protocol_state: ProtocolState::Closed,
            current_lobby: None,
            socket: None,
            client_id: "".to_string(),
            lobbies: vec![],
            is_listening_async: false,
            game_in_progress: None,
            supported_games: HashMap::new(),
            matching_supported_games: vec![],
            next_message_num: 0,
            on_message_received: None,
        }));
        Self {
            state,
            ip: None,
            port: None,
        }
    }

    pub fn get_socket_address(&self) -> String {
        if let (Some(ip), Some(port)) = (&self.ip, &self.port) {
            return format!("{}:{}", ip, port);
        }
        "".to_string()
    }

    pub fn get_protocol_state(&self) -> ProtocolState {
        self.state.lock().unwrap().protocol_state
    }

    pub fn get_supported_games(&self) -> Vec<(String, String)> {
        self.state.lock().unwrap().matching_supported_games.clone()
    }

    pub fn get_lobby_list(&self) -> Vec<Lobby> {
        self.state.lock().unwrap().lobbies.clone()
    }

    pub fn get_current_lobby(&self) -> Option<Lobby> {
        self.state.lock().unwrap().current_lobby.clone()
    }

    pub fn get_client_id(&self) -> String {
        self.state.lock().unwrap().client_id.clone()
    }

    pub fn get_game_state(&self) -> Option<Box<dyn GameState>> {
        let state_lock = self.state.lock().unwrap();
        if let Some(game) = &state_lock.game_in_progress {
            Some(game.get_game_state().clone())
        } else {
            None
        }
    }

    pub fn get_game_end_result(&self) -> Option<(bool, Option<String>)> {
        let state_lock = self.state.lock().unwrap();
        if let Some(game) = &state_lock.game_in_progress {
            Some(game.end_condition_met())
        } else {
            None
        }
    }

    pub fn on_message_received_callback(&self, callback: impl Fn() + Send + Sync + 'static) {
        let callback = Box::new(callback);
        self.state.lock().unwrap().on_message_received = Some(callback);
    }

    pub fn register_game<T: 'static + GameModule>(&mut self) {
        let game = Arc::new(T::new());
        self.state.lock().unwrap().supported_games.insert(game.get_metadata().get_game_type_id(), game);
    }

    pub fn request_lobby_list(&mut self) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        state_lock.previous_protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::GettingLobbies;
        drop(state_lock);

        self.send_message(build_client_headers(next_message_num, MessageType::LobbyListRequest));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn request_supported_games(&mut self) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        state_lock.previous_protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::GettingSupportedGames;
        drop(state_lock);

        self.send_message(build_client_headers(next_message_num, MessageType::SupportedGamesRequest));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn refresh_current_lobby(&mut self) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        state_lock.previous_protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::GettingLobbyInfo;
        drop(state_lock);

        self.send_message(build_client_headers(next_message_num, MessageType::LobbyInfoRequest));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn create_lobby(&mut self, game_type_id: &str) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        state_lock.previous_protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::CreatingLobby;
        drop(state_lock);

        self.send_message(build_create_lobby_request(next_message_num, game_type_id.to_string()));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn join_lobby(&mut self, lobby_id: &str) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        state_lock.previous_protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::JoiningLobby;
        drop(state_lock);

        self.send_message(build_join_lobby_request(next_message_num, lobby_id.to_string()));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn leave_lobby(&mut self) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        state_lock.previous_protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::LeavingLobby;
        drop(state_lock);

        self.send_message(build_client_headers(next_message_num, MessageType::LeaveLobbyRequest));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn start_game(&mut self, lobby_id: &String) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        state_lock.previous_protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::CreatingGameSession;
        drop(state_lock);
        self.send_message(build_start_game_request(next_message_num, lobby_id.clone()));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn make_move(&mut self, game_move: &dyn GameMove) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        drop(state_lock);

        self.send_message(build_move_request(next_message_num, game_move));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn return_to_lobby(&mut self) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        drop(state_lock);

        self.send_message(build_client_headers(next_message_num, MessageType::ReturnToLobbyRequest));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    pub fn connect(&mut self, ip: &String, port: &String) {
        // Return if socket is Some (has a value, already connected).
        if self.state.lock().unwrap().socket.is_some() {
            return;
        }

        // Set the game_protocol state and save the ip and port
        let mut state_lock = self.state.lock().unwrap();
        state_lock.previous_protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::Authenticating;
        self.ip = Some(ip.clone());
        self.port = Some(port.clone());

        // If it is None (unassigned, not connected), then try to connect.
        match TcpStream::connect(self.get_socket_address()) {
            Ok(tcp_stream) => {

                // Store the socket and send a connect request.
                let state_clone = self.state.clone();
                let socket = Arc::new(tcp_stream);
                state_lock.socket = Some(socket.clone());
                let connect_request = build_connect_request(state_lock.next_message_num, None);
                let socket = state_lock.socket.as_ref().unwrap().clone();
                drop(state_lock);

                self.send_message(connect_request);
                listen(socket, self.state.clone());
            }
            Err(e) => {
                state_lock.protocol_state = state_lock.protocol_state;
                state_lock.protocol_state = ProtocolState::Closed;
                println!("Connect error: {}", e);
            }
        }
    }

    pub fn disconnect(&mut self) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        let socket = state_lock.socket.as_ref().unwrap().clone();
        let next_message_num = state_lock.next_message_num;
        state_lock.protocol_state = state_lock.protocol_state;
        state_lock.protocol_state = ProtocolState::ClosingConnection;
        drop(state_lock);

        self.send_message(build_client_headers(next_message_num, MessageType::DisconnectRequest));
        if !self.state.lock().unwrap().is_listening_async {
            listen(socket, self.state.clone());
        }
    }

    fn send_message(&mut self, data: Vec<u8>) {
        let state_clone = self.state.clone();
        let mut state_lock = state_clone.lock().unwrap();
        if state_lock.socket.is_some() {
            if state_lock.next_message_num == u32::MAX {
                state_lock.next_message_num = 0;
            } else {
                state_lock.next_message_num += 1;
            }
            state_lock.socket.as_ref().unwrap().as_ref().write(data.as_slice());
        }
    }

    pub fn async_listen(&self) {
        let state_clone = self.state.clone();
        let socket_clone = self.state.lock().unwrap().socket.as_ref().unwrap().clone();
        thread::spawn(move || {
            state_clone.as_ref().lock().unwrap().is_listening_async = true;
            loop {
                if state_clone.as_ref().lock().unwrap().is_listening_async {
                    listen(socket_clone.clone(), state_clone.clone());
                } else {
                    break;
                }
            }
        });
    }

    pub fn stop_async_listen(&self) {
        self.state.lock().unwrap().is_listening_async = false;
    }
}

fn listen(socket: Arc<TcpStream>, state: Arc<Mutex<GameProtocolClientState>>) {
    if state.lock().unwrap().socket.is_some() {
        let mut buffer = [0; 4096];
        match socket.as_ref().read(&mut buffer) { // TODO should I set timeout?
            Ok(size) => {
                // If size is more than 0, then this is a legit message we are receiving.
                // If size is 0, then socket is closed, so formally shut it down.
                if size > 0 {
                    let (status_code, message_type, remainder) = parse_server_message_header(&buffer);
                    println!("From server: {:?}, {:?}, {}, {}", status_code, message_type, remainder.len(), size);

                    let mut state_lock = state.lock().unwrap();
                    match message_type {
                        MessageType::ConnectResponse => {
                            // Only accept the ConnectResponse if it was successful and this client_bin was in the correct state: Authenticating.
                            if matches!(status_code, StatusCode::Success) {
                                let response = parse_connect_response(remainder);
                                state_lock.client_id = response.client_id;
                                state_lock.protocol_state = ProtocolState::Idle;
                            }
                        }
                        MessageType::DisconnectResponse => {
                            if matches!(status_code, StatusCode::Success) {
                                state_lock.protocol_state = ProtocolState::Closed;
                                state_lock.socket = None;
                                state_lock.current_lobby = None;
                                state_lock.client_id = "".to_string();
                                state_lock.lobbies = vec![];
                                state_lock.matching_supported_games = vec![];
                                state_lock.next_message_num = 0;
                            }
                        }
                        MessageType::LobbyListResponse => {
                            if matches!(status_code, StatusCode::Success) {
                                let data = parse_lobby_list_response(remainder);
                                state_lock.protocol_state = ProtocolState::Idle;
                                state_lock.lobbies = data.lobbies;
                            }
                        }
                        MessageType::SupportedGamesResponse => {
                            if matches!(status_code, StatusCode::Success) {
                                let data = parse_supported_games_response(remainder);

                                // Compare list of server_bin supported games with client_bin supported games.
                                // Collect the matching games and store them since these are the ones the client_bin should only be able to create lobbies for and join.
                                let mut matching_games = vec![];
                                let supported_games = &state_lock.supported_games;
                                for server_game_id in data.games.iter() {
                                    if supported_games.contains_key(server_game_id) {
                                        matching_games.push((supported_games.get(server_game_id).unwrap().get_metadata().game_title.clone(), server_game_id.clone()));
                                    }
                                }
                                state_lock.protocol_state = ProtocolState::Idle;
                                state_lock.matching_supported_games = matching_games;
                            }
                        }
                        MessageType::LobbyInfoResponse => {
                            if matches!(status_code, StatusCode::Success) {
                                let data = parse_lobby_info_response(remainder);
                                state_lock.protocol_state = ProtocolState::InLobby;
                                state_lock.current_lobby = Some(data.lobby);
                            }
                        }
                        MessageType::LeaveLobbyResponse => {
                            if matches!(status_code, StatusCode::Success) {
                                state_lock.protocol_state = ProtocolState::Idle;
                                state_lock.current_lobby = None;
                            }
                        }
                        MessageType::GameStateResponse => {
                            let game_type_id = state_lock.current_lobby.as_ref().unwrap().game_metadata.get_game_type_id();
                            let response = parse_game_state_response(remainder);

                            if matches!(state_lock.protocol_state, ProtocolState::CreatingGameSession) || matches!(state_lock.protocol_state, ProtocolState::InLobby) {
                                let mut new_game = state_lock.supported_games.get(&game_type_id).unwrap().init_new();
                                new_game.set_game_state(response);
                                state_lock.game_in_progress = Some(new_game);
                                state_lock.protocol_state = ProtocolState::GameRunning;
                            } else if state_lock.game_in_progress.is_some() {
                                state_lock.game_in_progress.as_mut().unwrap().set_game_state(response);
                            }
                        }
                        MessageType::UnsolicitedMessage => {}
                        MessageType::MissingMessageResponse => {}
                        MessageType::ProtocolError => {
                            // TODO handle all kinds of errors. If error is encountered, revert to previous game_protocol state. Need to keep track of that.
                            //  For example, if we are creating a lobby and receive a game_protocol error, then it's going to
                            //  be an error about creating the lobby. So handle it appropriately.
                            state_lock.protocol_state = state_lock.previous_protocol_state;
                        }
                        MessageType::Unsupported => {}
                        _ => {} // Default and Unsupported. Do nothing if we get a message unsupported on the client_bin side.
                    };

                    // Run on message received callback if it was set
                    if let Some(callback) = &state_lock.on_message_received {
                        callback();
                    }
                }
            }
            Err(e) => {
                println!("Listen error: {}", e);
            }
        };
    }
}
