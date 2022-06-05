use serde::{Serialize, Deserialize};
use crate::game_module::{GameMetadata};

/*
    Contains data structures that are shared between client and server
 */


// Lobby object to store in server and display on client side
#[derive(Serialize, Deserialize, Clone)]
pub struct Lobby {
    pub id: String,
    pub owner: String,
    pub player_ids: Vec<String>,
    pub game_started: bool,
    pub game_metadata: GameMetadata,
}

// Functions for Lobby objects
impl Lobby {
    // Determine if lobby is full
    pub fn is_full(&self) -> bool {
        let connected_clients = self.player_ids.len();
        connected_clients >= self.game_metadata.max_players
    }
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

// Represents data for connecting to the server
pub struct ConnectRequest<T> where T: ConnectRequestAuth {
    auth_data: T,
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


// A series of simple structs as wrappers around message specific data. Not much to explain.


#[derive(Serialize, Deserialize)]
pub struct JoinLobbyRequest {
    pub lobby_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateLobbyRequest {
    pub game_type_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StartGameRequest {
    pub lobby_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct LobbyInfoResponse {
    pub lobby: Lobby,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectResponse {
    pub client_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MissingMessageResponse {
    pub missing_message_ids: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UnsolicitedMessage {
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct LobbyListResponse {
    pub lobbies: Vec<Lobby>,
}

#[derive(Serialize, Deserialize)]
pub struct SupportedGamesResponse {
    pub games: Vec<String>,
}


