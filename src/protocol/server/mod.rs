use std::net::{Shutdown, TcpListener, TcpStream};
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::hash::Hash;
use std::io::{Read, Write};
use std::rc::Rc;
use std::thread;
use uuid::Uuid;
use crate::enums::{MessageType, StatusCode};
use crate::game_module::{GameModule};
use crate::server::server_message_utils::{build_connect_response, build_game_state_response, build_lobby_info_response, build_lobby_list_response, build_server_headers, build_supported_game_response, parse_client_message_header, parse_connect_request, parse_create_lobby_request, parse_join_lobby_request, parse_move_request, parse_start_game_request};
use crate::shared_data::Lobby;

mod server_message_utils;

// Give TcpStream its own send_message function as a wrapper around the socket's write function.
pub trait SocketSend {
    fn send_message(&self, data: Vec<u8>);
}
impl SocketSend for TcpStream {
    fn send_message(&self, data: Vec<u8>) {
        self.clone().write(data.as_slice());
    }
}

// Simple struct for data specific to each client_bin.
pub struct Client {
    socket: Arc<TcpStream>,
    id: String,
    lobby_id: Option<String>,
    next_message_id: u32
}

struct GameProtocolServerState {
    clients: HashMap<String, Client>,
    lobbies: HashMap<String, Lobby>,
    supported_games: HashMap<String, Arc<dyn GameModule>>,
    games_in_progress: HashMap<String, Box<dyn GameModule>>
}

pub struct GameProtocolServer {
    state: Arc<Mutex<GameProtocolServerState>>,
    listener: Option<TcpListener>,
    ip: String,
    port: String
}

impl GameProtocolServer {
    pub fn new(ip: &str, port: &str) -> Self {
        Self {
            state: Arc::new(Mutex::new(GameProtocolServerState{
                clients: HashMap::new(),
                lobbies: HashMap::new(),
                supported_games: HashMap::new(),
                games_in_progress: HashMap::new()
            })),
            listener: None,
            ip: ip.to_string(),
            port: port.to_string()
        }
    }

    pub fn register_game<T: 'static + GameModule>(&self) {
        let game = Arc::new(T::new());
        let mut state_lock = self.state.lock().unwrap();
        state_lock.supported_games.insert(game.get_metadata().get_game_type_id(),  game);
    }

    pub fn start(&mut self) {
        println!("Starting server_bin...");
        // Binds a TCP listener to an IP address and port number, creating a socket address.
        match TcpListener::bind(format!("{}:{}", self.ip, self.port)) {
            Ok(listener) => {
                self.listener = Some(listener);
                println!("Server listening on port {}", self.port);
                self.start_server_loop();
            },
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }

    // Listen for any incoming client_bin connections. For each one, split off into a new thread.
    fn start_server_loop(&mut self) {
        // Wait for incoming connection attempts.
        for stream in self.listener.as_ref().unwrap().incoming() {
            // Make sure getting the stream is successful.
            match stream {
                Ok(stream) => {
                    println!("New connection: {}", stream.peer_addr().unwrap());
                    self.listen_to_client(stream);
                },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
    }

    fn listen_to_client(&self, stream: TcpStream) {
        let state_clone = self.state.clone();
        let client_socket = Arc::new(stream);
        thread::spawn(move|| {
            // Initialize client_bin ID as an empty string, indicating that it does not have an active session yet.
            let mut client_id = String::from("");
            loop {
                let mut buffer = [0; 1024]; // TODO probably increase this 1024 byte buffer

                // Read stream data into the buffer.
                match client_socket.as_ref().read(&mut buffer) {
                    Ok(size) => {
                        // If size is more than 0, then this is a legit message we are receiving.
                        // If size is 0, then socket is closed, so formally shut it down.
                        if size > 0 {
                            println!("Message received. Processing...");
                            let (message_id, message_type, remainder) = parse_client_message_header(&buffer);
                            println!("id: {}, type: {:?}, data size: {}", message_id, message_type, size);

                            // If client_bin is not authenticated by the server_bin and stored as a connected client_bin,
                            // then server_bin will only accept ConnectRequests and send client_bin an error otherwise.
                            let is_id_empty = client_id.is_empty();
                            if is_id_empty && matches!(message_type, MessageType::ConnectRequest) {
                                // If authentication is successful, add client_bin to the server_bin.
                                let connect_message = parse_connect_request(remainder);
                                if connect_message.authenticate() {
                                    // Create new client_bin struct and add it to the hash map
                                    let new_client_id = Uuid::new_v4().to_string();
                                    let new_client = Client {
                                        socket: client_socket.clone(),
                                        id: new_client_id.clone(),
                                        lobby_id: None,
                                        next_message_id: 0
                                    };
                                    client_id = new_client_id.clone();
                                    state_clone.lock().unwrap().clients.insert(new_client_id.clone(), new_client);
                                    client_socket.send_message(build_connect_response(StatusCode::Success, new_client_id));
                                }
                            } else if is_id_empty {
                                // Send client_bin an error if it does not have an ID, indicating that it has not been added to the server_bin as an active client_bin.
                                client_socket.send_message(build_server_headers(StatusCode::NoActiveSession, MessageType::ProtocolError));
                                continue;
                            }

                            // TODO is it possible to generalize some of the handling code for each message? Like can stuff for lobbies be generalized?
                            // If the client_bin ID is not an empty string, indicating it has an active session, then we handle any type of message from the client_bin.
                            match message_type {
                                MessageType::DisconnectRequest => {
                                    // Break out of the listening loop and handle clean up at the bottom of this function.
                                    let response = build_server_headers(StatusCode::Success, MessageType::DisconnectResponse);
                                    client_socket.send_message(response);
                                    break;
                                }
                                MessageType::LobbyListRequest => {
                                    let lobbies = state_clone.lock().unwrap().lobbies.clone().into_values().collect();
                                    let response = build_lobby_list_response(StatusCode::Success, &lobbies);
                                    client_socket.send_message(response);
                                }
                                MessageType::CreateLobbyRequest => {
                                    let create_lobby_request = parse_create_lobby_request(remainder);
                                    let mut state_lock = state_clone.lock().unwrap();
                                    let mut state_ref = state_lock.deref_mut();

                                    // Check if server supports the game. Otherwise send an error.
                                    if state_ref.supported_games.contains_key(&create_lobby_request.game_type_id) {
                                        // Create new lobby
                                        let client = state_ref.clients.get_mut(&client_id).unwrap();
                                        let new_lobby_id = Uuid::new_v4().to_string();
                                        let new_lobby = Lobby {
                                            owner: client.id.clone(),
                                            id: new_lobby_id.clone(),
                                            player_ids: vec![client.id.clone()],
                                            game_started: false,
                                            game_metadata: state_ref.supported_games.get(&create_lobby_request.game_type_id).unwrap().get_metadata().clone()
                                        };
                                        client.lobby_id = Some(new_lobby_id.clone());
                                        state_ref.lobbies.insert(new_lobby_id, new_lobby.clone());
                                        client_socket.send_message(build_lobby_info_response(StatusCode::Success, new_lobby.clone()));
                                    } else {
                                        client_socket.send_message(build_server_headers(StatusCode::UnsupportedGame, MessageType::ProtocolError));
                                    }
                                }
                                MessageType::SupportedGamesRequest => {
                                    let state_lock = state_clone.lock().unwrap();
                                    let games = state_lock.supported_games.keys().cloned().collect();
                                    let response = build_supported_game_response(StatusCode::Success, &games);
                                    client_socket.send_message(response);
                                }
                                MessageType::JoinLobbyRequest => {
                                    // TODO if lobby is full or already started, don't allow client to join
                                    let mut state_lock = state_clone.lock().unwrap();
                                    let state_ref = state_lock.deref_mut();

                                    // Make sure client_bin isn't already in a lobby
                                    let client = state_ref.clients.get_mut(&client_id).unwrap();
                                    if client.lobby_id.is_none() {
                                        let join_request = parse_join_lobby_request(remainder);
                                        let lobby = state_ref.lobbies.get_mut(&join_request.lobby_id).unwrap();

                                        // If lobby isn't full, add the client_bin to the lobby, send other connected clients updated lobby info, and send client_bin lobby info
                                        if !lobby.is_full() {
                                            // Cache original list of current clients so we can later send them all an updated lobby state
                                            let original_client_ids = lobby.player_ids.clone();

                                            // Update lobby and client_bin
                                            lobby.player_ids.push(client_id.clone());
                                            client.lobby_id = Some(join_request.lobby_id.clone());

                                            // Build response to send to all clients in lobby, including the newly added one.
                                            let response = build_lobby_info_response(StatusCode::Success, lobby.clone());
                                            client_socket.send_message(response.clone());

                                            // Send all clients an updated lobby state.
                                            for client_id in original_client_ids.iter() {
                                                state_ref.clients.get(client_id).unwrap().socket.send_message(response.clone());
                                            }
                                        } else if lobby.game_started {
                                            client_socket.send_message(build_server_headers(StatusCode::GameStarted, MessageType::ProtocolError));
                                        } else {
                                            client_socket.send_message(build_server_headers(StatusCode::LobbyFull, MessageType::ProtocolError));
                                        }
                                    }
                                }
                                MessageType::LobbyInfoRequest => {
                                    let mut state_lock = state_clone.lock().unwrap();
                                    let state_ref = state_lock.deref_mut();

                                    // Make sure client_bin is in a lobby first. If so, send them the current info.
                                    let client = state_ref.clients.get_mut(&client_id).unwrap();
                                    if let Some(lobby_id) = client.lobby_id.clone() {
                                        let lobby = state_ref.lobbies.get(&lobby_id).unwrap();
                                        client_socket.send_message(build_lobby_info_response(StatusCode::Success, lobby.clone()));
                                    } else {
                                        // If not in a lobby, send a lobby error with NotInLobby
                                        client_socket.send_message(build_server_headers(StatusCode::NotInLobby, MessageType::ProtocolError));
                                    }
                                }
                                MessageType::LeaveLobbyRequest => {
                                    // If the client_bin is in a lobby, then leave it.
                                    let mut state_lock = state_clone.lock().unwrap();
                                    let state_ref = state_lock.deref_mut();

                                    let client = state_ref.clients.get_mut(&client_id).unwrap();
                                    if client.lobby_id.is_some() {
                                        // Find the lobby the client_bin is in
                                        let lobby_id = client.lobby_id.as_ref().unwrap().clone();
                                        let mut found_lobby = state_ref.lobbies.get_mut(&lobby_id).unwrap();

                                        // Find the position in which the player is in the lobby and remove it
                                        let client_position = found_lobby.player_ids.iter().position(|id|
                                            id.eq(&client.id)
                                        ).unwrap();
                                        found_lobby.player_ids.remove(client_position);
                                        client.lobby_id = None;

                                        // If the lobby is empty, remove it from the server
                                        if found_lobby.player_ids.len() == 0 {
                                            state_lock.lobbies.remove(&lobby_id);
                                        } else {
                                            // If client is owner, transfer ownership to another client
                                            if found_lobby.owner.eq(&client_id) {
                                                found_lobby.owner = found_lobby.player_ids[0].clone();
                                            }

                                            // Otherwise send remaining clients an updated lobby state
                                            let response = build_lobby_info_response(StatusCode::Success, found_lobby.clone());

                                            // Send all clients an updated lobby state.
                                            for id in found_lobby.player_ids.iter() {
                                                state_ref.clients.get(id).unwrap().socket.send_message(response.clone());
                                            }
                                        }

                                        // Send the client_bin a LeaveLobbyResponse, confirming that the server_bin has removed the client_bin from the lobby
                                        client_socket.send_message(build_server_headers(StatusCode::Success, MessageType::LeaveLobbyResponse));
                                    }
                                }
                                MessageType::StartGameRequest => {
                                    let start_request = parse_start_game_request(remainder);
                                    let mut state_lock = state_clone.lock().unwrap();
                                    let state_ref = state_lock.deref_mut();
                                    let client = state_ref.clients.get(&client_id).unwrap();

                                    // Check if client is in a lobby first.
                                    if let Some(lobby_id) = &client.lobby_id {
                                        let lobby = state_ref.lobbies.get_mut(lobby_id).unwrap();
                                        let player_req_met =
                                            lobby.player_ids.len() >= lobby.game_metadata.min_required_players &&
                                            lobby.player_ids.len() <= lobby.game_metadata.max_players;

                                        // Start game if player requirement is met, requested lobby is the one the client is in, and if the client is the owner of the lobby
                                        if start_request.lobby_id.eq(lobby_id) && lobby.owner.eq(&client_id) && player_req_met {
                                            let mut new_game = state_ref.supported_games.get(&lobby.game_metadata.get_game_type_id()).unwrap().init_new();

                                            for id in lobby.player_ids.iter() {
                                                new_game.add_player(id.clone());
                                            }

                                            // Tie game session with lobby ID
                                            state_ref.games_in_progress.insert(lobby_id.clone(), new_game);

                                            for id in lobby.player_ids.iter() {
                                                let game = state_ref.games_in_progress.get(lobby_id).unwrap();
                                                let game_state = build_game_state_response(StatusCode::Success, game.get_game_state());
                                                state_ref.clients.get(id).unwrap().socket.send_message(game_state);
                                            }


                                            // Set lobby as the game is in progress
                                            lobby.game_started = true;
                                        }
                                    } else {
                                        // If not in lobby, send error
                                        client_socket.send_message(build_server_headers(StatusCode::NotInLobby, MessageType::ProtocolError));
                                    }

                                }
                                MessageType::MoveRequest => {
                                    let client_move = parse_move_request(remainder);
                                    let mut state_lock = state_clone.lock().unwrap();
                                    let state_ref = state_lock.deref_mut();
                                    let client = state_ref.clients.get_mut(&client_id).unwrap();

                                    // If client wasn't in a lobby, it's definitely not in a game session.
                                    // Also lobby ID is used for the game session hash map too.
                                    if let Some(lobby_id) = &client.lobby_id {
                                        if let Some(game) = state_ref.games_in_progress.get_mut(lobby_id) {
                                            let game_ended = game.end_condition_met().0;
                                            let is_move_valid = game.is_valid_move(&client_move);
                                            if is_move_valid && !game_ended {
                                                println!("valid move? {}", game.is_valid_move(&client_move));
                                                game.apply_move(&client_move);

                                                let game_state = build_game_state_response(StatusCode::Success, game.get_game_state());
                                                for id in state_ref.lobbies.get(lobby_id).unwrap().player_ids.iter() {
                                                    state_ref.clients.get(id).unwrap().socket.send_message(game_state.clone());
                                                }
                                            } else if game_ended {
                                                // If client tries to make a move but game is over, send a GameOver protocol error.
                                                client.socket.send_message(build_server_headers(StatusCode::GameOver, MessageType::ProtocolError));
                                            } else if is_move_valid {
                                                // If client's move is invalid, then return an error.
                                                client.socket.send_message(build_server_headers(StatusCode::InvalidMove, MessageType::ProtocolError));
                                            }
                                        }
                                    }
                                }
                                MessageType::ReturnToLobbyRequest => {
                                    // TODO in progress.
                                    //  Send return to lobby response to requesting client.
                                    let mut state_lock = state_clone.lock().unwrap();
                                    let state_ref = state_lock.deref_mut();
                                    let client = state_ref.clients.get_mut(&client_id).unwrap();

                                    if let Some(lobby_id) = &client.lobby_id {
                                        if let Some(game) = state_ref.games_in_progress.get_mut(lobby_id) {
                                            game.remove_player(client.id.clone());
                                            if game.get_player_num() == 0 {
                                                state_ref.games_in_progress.remove(lobby_id);
                                                state_ref.lobbies.get_mut(lobby_id).unwrap().game_started = false;
                                            }
                                            let lobby = state_ref.lobbies.get(lobby_id).unwrap();
                                            client.socket.send_message(build_lobby_info_response(StatusCode::Success, lobby.clone()));
                                        }
                                    }
                                }
                                MessageType::GameStateRequest => {}
                                MessageType::ResendMessageRequest => {}
                                MessageType::UnsolicitedMessage => {}
                                MessageType::Unsupported => {}
                                _ => {} // Default and Unsupported. Do nothing.
                            };
                            println!("Done");
                        } else {
                            println!("Shutting down...");
                            client_socket.shutdown(Shutdown::Both);
                            break;
                        }
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                    }
                }

            }
            // When the listening loop exits, do clean up.
            // TODO Remove client from any game_module session it's in.
            // Remove client_bin from any lobby it's in.
            // Remove client_bin from client_bin list on server_bin.
            if !client_id.is_empty() {
                let mut state_lock = state_clone.lock().unwrap();

                // Need to deref_mut this and get mutable references to clients and lobbies to prevent multiple mutable borrows of state_lock
                let state_ref = state_lock.deref_mut();
                let clients = &mut state_ref.clients;
                let lobbies = &mut state_ref.lobbies;

                // Get client_bin being handled in this thread and check if it is in a lobby.
                let client = clients.get_mut(&client_id).unwrap();
                if client.lobby_id.is_some() {
                    // Get the lobby the client_bin is in, find where the client_bin is in the lobby player_ids list and remove it.
                    let lobby_id = client.lobby_id.as_ref().unwrap().clone();
                    let mut found_lobby = lobbies.get_mut(&lobby_id).unwrap();
                    let client_position = found_lobby.player_ids.iter().position(|id|
                        id.eq(&client_id)
                    ).unwrap();
                    found_lobby.player_ids.remove(client_position);
                    client.lobby_id = None; // Set client_bin lobby_id to None just to be safe even though client_bin is removed later.

                    // If the lobby is empty, remove it from the server_bin
                    if found_lobby.player_ids.len() == 0 {
                        lobbies.remove(&lobby_id);
                    }
                }
                clients.remove(&client_id);
            }
        });
    }
}