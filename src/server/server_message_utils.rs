use std::mem::size_of;
use serde_json;
use crate::common_message_utils::{build_message_body, parse_message_type};
use crate::enums::{ServerError, MessageType, StatusCode};
use crate::game_module::{GameState};
use crate::shared_data::{ConnectRequest, ConnectResponse, Lobby, LobbyInfoResponse, LobbyListResponse, MissingMessageResponse, NoAuth, SupportedGamesResponse, UnsolicitedMessage};

/*
    Full of helper functions to parse client requests and build server responses.
    A lot of these could probably be made to use generics.
    to sort out a refactor. This all still works as intended.
    Functions should be self explanatory: build and parse message types.
 */


// Parse client message headers, returning the message ID, message type, and remaining data
pub fn parse_client_message_header(raw_message: &[u8]) -> (u32, MessageType, &[u8]) {
    // Message sequence ID.
    let (id_bytes, remainder) = raw_message.split_at(size_of::<u32>());
    let message_id = u32::from_be_bytes(id_bytes.try_into().unwrap());

    // Message type.
    let (message_type, remainder) = parse_message_type(remainder);
    (message_id, message_type, remainder)
}

pub fn parse_connect_request(_: &[u8]) -> ConnectRequest<NoAuth> {
    ConnectRequest::new(NoAuth {})
}

// Build the headers for server message: status code and message type.
pub fn build_server_headers(status_code: StatusCode, message_type: MessageType) -> Vec<u8> {
    let mut byte_vec = vec![];
    byte_vec.extend_from_slice(&(status_code as u16).to_be_bytes()); // status code
    byte_vec.extend_from_slice(&(message_type as u16).to_be_bytes()); // message type
    byte_vec
}

pub fn build_missing_message_response(missing_message_ids: Vec<u32>) -> Vec<u8> {
    let mut byte_vec = build_server_headers(StatusCode::MessageSequenceError, MessageType::MissingMessageResponse);
    let missing_messages = MissingMessageResponse { missing_message_ids };
    let serialized_response = serde_json::to_string(&missing_messages).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(serialized_response)));
    byte_vec
}

pub fn build_unsolicited_message(status_code: StatusCode, message: &str) -> Vec<u8> {
    let mut byte_vec = build_server_headers(status_code, MessageType::UnsolicitedMessage);
    let unsolicited_message = UnsolicitedMessage { message: message.to_string() };
    let serialized = serde_json::to_string(&unsolicited_message).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(serialized)));
    byte_vec
}

pub fn build_connect_response(status_code: StatusCode, client_id: String) -> Vec<u8> {
    let mut byte_vec = build_server_headers(status_code, MessageType::ConnectResponse);
    let connect_response = ConnectResponse { client_id };
    let serialized_response = serde_json::to_string(&connect_response).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(serialized_response)));
    byte_vec
}

pub fn build_lobby_list_response(status_code: StatusCode, lobbies: &Vec<Lobby>) -> Vec<u8> {
    let mut byte_vec = build_server_headers(status_code, MessageType::LobbyListResponse);
    let lobby_list = LobbyListResponse { lobbies: lobbies.to_vec() };
    let serialized_lobbies = serde_json::to_string(&lobby_list).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(serialized_lobbies)));
    byte_vec
}

pub fn build_supported_game_response(status_code: StatusCode, games: &Vec<String>) -> Vec<u8> {
    let mut byte_vec = build_server_headers(status_code, MessageType::SupportedGamesResponse);
    let supported_game_response = SupportedGamesResponse { games: games.clone() };
    let serialized_games = serde_json::to_string(&supported_game_response).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(serialized_games)));
    byte_vec
}

pub fn build_lobby_info_response(status_code: StatusCode, lobby: Lobby) -> Vec<u8> {
    let mut byte_vec = build_server_headers(status_code, MessageType::LobbyInfoResponse);
    let lobby_response = LobbyInfoResponse { lobby: lobby.clone() };
    let serialized_lobby = serde_json::to_string(&lobby_response).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(serialized_lobby)));
    byte_vec
}

pub fn build_game_state_response(status_code: StatusCode, game_state: &dyn GameState) -> Vec<u8> {
    let mut byte_vec = build_server_headers(status_code, MessageType::GameStateResponse);
    let serialized = serde_json::to_string(game_state).unwrap();
    byte_vec.extend_from_slice(&build_message_body(Some(serialized)));
    byte_vec
}

// Based on the error enum received, build an error response to send to the client.
pub fn build_server_error_response(e: ServerError) -> Vec<u8> {
     match e {
         ServerError::ChecksumError => {
            build_server_headers(StatusCode::DataIntegrityError, MessageType::ProtocolError)
        }
         ServerError::BodySizeError => {
            build_server_headers(StatusCode::DataIntegrityError, MessageType::ProtocolError)
        }
         ServerError::BytesToStringError => {
             build_server_headers(StatusCode::DataParseError, MessageType::ProtocolError)
        }
         ServerError::DeserializeError => {
             build_server_headers(StatusCode::MalformedBody, MessageType::ProtocolError)
        }
    }
}