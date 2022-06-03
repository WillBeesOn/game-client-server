use serde::Deserialize;
use crate::common_message_utils::{build_message_body, parse_message_payload, parse_message_type, parse_status_code};
use crate::enums::{MessageType, StatusCode};
use crate::game_module::{GameMove, GameState};
use crate::shared_data::{ConnectResponse, CreateLobbyRequest, JoinLobbyRequest, LobbyInfoResponse, LobbyListResponse, MissingMessageResponse, StartGameRequest, SupportedGamesResponse, UnsolicitedMessage};

/*
    Contains helpers for building client requests and parsing server responses.
    A lot of these could probably be made to use generics, but currently don't have enough time
    to sort out a refactor. This all still works as intended.
    Functions should be self explanatory: build and parse message types.
 */

// Build the basic client message fields into a byte vector to send.
// Handles message sequence and message type.
pub fn build_client_headers(next_in_sequence: u32, message_type: MessageType) -> Vec<u8> {
    let mut byte_vec = vec![];
    byte_vec.extend_from_slice(&next_in_sequence.to_be_bytes());
    byte_vec.extend_from_slice(&(message_type as u16).to_be_bytes());
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

// Parse server message headers, returning them and the remaining bytes of data
pub fn parse_server_message_header(raw_message: &[u8]) -> (StatusCode, MessageType, &[u8]) {
    // Message sequence ID.
    let (status_code, remainder) = parse_status_code(raw_message);

    // Message type.
    let (message_type, remainder) = parse_message_type(remainder);
    (status_code, message_type, remainder)
}

pub fn parse_connect_response(raw_message: &[u8]) -> ConnectResponse {
    let data_string = parse_message_payload(raw_message);
    let data: ConnectResponse = serde_json::from_slice(data_string.as_bytes()).unwrap();
    data
}

pub fn parse_lobby_list_response(raw_message: &[u8]) -> LobbyListResponse {
    let data_string = parse_message_payload(raw_message);
    let data: LobbyListResponse = serde_json::from_slice(data_string.as_bytes()).unwrap();
    data
}

pub fn parse_lobby_info_response(raw_message: &[u8]) -> LobbyInfoResponse {
    let data_string = parse_message_payload(raw_message);
    let data: LobbyInfoResponse = serde_json::from_slice(data_string.as_bytes()).unwrap();
    data
}

pub fn parse_supported_games_response(raw_message: &[u8]) -> SupportedGamesResponse {
    let data_string = parse_message_payload(raw_message);
    let data: SupportedGamesResponse = serde_json::from_slice(data_string.as_bytes()).unwrap();
    data
}

pub fn parse_game_state_response(raw_message: &[u8]) -> Box<dyn GameState> {
    let data_string = parse_message_payload(raw_message);
    let data: Box<dyn GameState> = serde_json::from_slice(data_string.as_bytes()).unwrap();
    data
}

pub fn parse_missing_message_response(raw_message: &[u8]) -> MissingMessageResponse {
    let data_string = parse_message_payload(raw_message);
    let data: MissingMessageResponse = serde_json::from_slice(data_string.as_bytes()).unwrap();
    data
}

pub fn parse_unsolicited_message(raw_message: &[u8]) -> UnsolicitedMessage {
    let data_string = parse_message_payload(raw_message);
    let data: UnsolicitedMessage = serde_json::from_slice(data_string.as_bytes()).unwrap();
    data
}
