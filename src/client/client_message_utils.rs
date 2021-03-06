use crate::common_message_utils::{build_message_body, parse_message_type, parse_status_code};
use crate::enums::{MessageType, StatusCode};
use crate::game_module::{GameMove};
use crate::shared_data::{CreateLobbyRequest, JoinLobbyRequest, StartGameRequest};

/*
    Contains helpers for building client requests and parsing server responses.
    A lot of these could probably be made to use generics.
    to sort out a refactor. This all still works as intended.
    Functions should be self explanatory: build and parse message types.
 */

// Parse server message headers, returning them and the remaining bytes of data
pub fn parse_server_message_header(raw_message: &[u8]) -> (StatusCode, MessageType, &[u8]) {
    // Message sequence ID.
    let (status_code, remainder) = parse_status_code(raw_message);

    // Message type.
    let (message_type, remainder) = parse_message_type(remainder);
    (status_code, message_type, remainder)
}

// Build the basic client message fields into a byte vector to send.
// Handles message sequence and message type.
pub fn build_client_headers(next_in_sequence: u32, message_type: MessageType) -> Vec<u8> {
    let mut byte_vec = vec![];
    byte_vec.extend_from_slice(&next_in_sequence.to_be_bytes()); // message id
    byte_vec.extend_from_slice(&(message_type as u16).to_be_bytes()); // message type
    byte_vec
}

pub fn build_connect_request(next_in_sequence: u32, body: Option<String>) -> Vec<u8> {
    let mut byte_vec = build_client_headers(next_in_sequence, MessageType::ConnectRequest);
    byte_vec.extend_from_slice(&build_message_body(body));
    byte_vec
}

pub fn build_join_lobby_request(next_in_sequence: u32, lobby_id: String) -> Vec<u8> {
    let mut byte_vec = build_client_headers(next_in_sequence, MessageType::JoinLobbyRequest);
    let join_json = serde_json::to_string(&JoinLobbyRequest { lobby_id }).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(join_json)));
    byte_vec
}

pub fn build_create_lobby_request(next_in_sequence: u32, game_type_id: String) -> Vec<u8> {
    let mut byte_vec = build_client_headers(next_in_sequence, MessageType::CreateLobbyRequest);
    let create_lobby_json = serde_json::to_string(&CreateLobbyRequest { game_type_id }).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(create_lobby_json)));
    byte_vec
}

pub fn build_start_game_request(next_in_sequence: u32, lobby_id: String) -> Vec<u8> {
    let mut byte_vec = build_client_headers(next_in_sequence, MessageType::StartGameRequest);
    let start_game = serde_json::to_string(&StartGameRequest { lobby_id }).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(start_game)));
    byte_vec
}

pub fn build_move_request(next_in_sequence: u32, game_move: &dyn GameMove) -> Vec<u8> {
    let mut byte_vec = build_client_headers(next_in_sequence, MessageType::MoveRequest);
    let serialized = serde_json::to_string(game_move).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(serialized)));
    byte_vec
}