use std::io::Write;
use std::net::TcpStream;
use std::rc::Rc;
use std::sync::Arc;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::game_module::GameMetadata;

#[derive(Serialize, Deserialize, Clone)]
pub struct Lobby {
    pub id: String,
    pub player_ids: Vec<String>,
    pub max_players: u8,
    pub game_title: String,
    pub owner: String
}

impl Lobby {
    pub fn is_full(&self) -> bool {
        let connected_clients = self.player_ids.len();
        connected_clients >= self.max_players as usize
    }
}

#[derive(Serialize, Deserialize)]
pub struct LobbyInfoResponse {
    pub lobby: Lobby
}

// To extend ConnectRequest to use authentication data, then create a struct the implements this trait.
pub trait ConnectRequestAuth {
    fn authenticate(&self) -> bool; // Returns if the user is authenticated given the data within the struct.
}

// Default. ConnectionRequest requires no authentication.
pub struct NoAuth {}
impl ConnectRequestAuth for NoAuth {
    fn authenticate(&self) -> bool {
        true
    }
}

// Represents data for connecting to the server_bin
pub struct ConnectRequest<T> where T: ConnectRequestAuth {
    auth_data: T
}

// Generic impl for ConnectRequest to use authenticate function from any custom auth struct
impl<T> ConnectRequest<T> where T: ConnectRequestAuth {
    pub fn new(auth_data: T) -> Self {
        Self {
            auth_data
        }
    }

    pub fn authenticate(&self) -> bool {
        self.auth_data.authenticate()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectResponse {
    pub client_id: String
}

// Represents data to join a lobby. Just contains the lobby id.
#[derive(Serialize, Deserialize)]
pub struct JoinLobbyRequest {
    pub lobby_id: String
}

#[derive(Serialize, Deserialize)]
pub struct LobbyListResponse {
    pub lobbies: Vec<Lobby>
}

#[derive(Serialize, Deserialize)]
pub struct SupportedGamesResponse {
    pub games: Vec<GameMetadata>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateLobbyRequest {
    pub game_key: String
}


#[derive(Serialize, Deserialize, Clone)]
pub struct StartGameRequest {
    pub lobby_id: String
}

