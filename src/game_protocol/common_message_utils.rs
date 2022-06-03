use std::mem::size_of;
use std::str;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use crate::enums::{MessageType, StatusCode};

/*
    Contains message parsing functions common between client and server.
 */


// Parse size and body of message into u32 and JSON string.
// Throw errors if the body checksum or size in bytes between
// incoming message data and local calculations does not match
pub fn parse_message_payload(data: &[u8]) -> Result<String, &'static str> {
    // Data size
    let (size_bytes, remainder) = data.split_at(size_of::<u32>());
    let size = u32::from_be_bytes(size_bytes.try_into().unwrap());

    // Get body and compare checksum
    let mut body = "";
    if size > 0 {
        // Extract remote checksum
        let (checksum_bytes, remainder) = remainder.split_at(size_of::<u32>());
        let remote_checksum = u32::from_be_bytes(checksum_bytes.try_into().unwrap());

        // Extract data and get checksum for the data
        let (data_bytes, _) = remainder.split_at(size as usize);
        let local_checksum = crc32fast::hash(&data_bytes);

        // If the checksums don't match, throw error. Otherwise parse the data into a string
        if remote_checksum != local_checksum {
            return Err("Checksums do not match.");
        }

        // Throw error if size described in message is not equal to the number of bytes of the body
        if size as usize > data_bytes.len() {
            return Err("Size of body data does not match.");
        }

        body = str::from_utf8(data_bytes).unwrap();
    }
    Ok(body.to_string())
}

// Build the body into a byte vector to send.
pub fn build_message_body(body: Option<String>) -> Vec<u8> {
    let mut byte_vec = vec![];

    if let Some(data) = body {
        let data_bytes = data.as_bytes();
        // If data size is larger than we can transfer, return an empty vec to indicate such.
        if data_bytes.len() > u32::MAX as usize {
            return byte_vec;
        }
        byte_vec.extend_from_slice(&(data_bytes.len() as u32).to_be_bytes());
        byte_vec.extend_from_slice(&crc32fast::hash(&data_bytes).to_be_bytes()); // Create checksum
        byte_vec.extend_from_slice(&data_bytes);
    } else {
        byte_vec.extend_from_slice(&0_u32.to_be_bytes());
    }
    byte_vec
}

// Generic function for parsing byte array into a data type
pub fn parse_message_data<'a, T>(raw_message: &[u8]) -> Result<T, &'static str> where T: DeserializeOwned {
    return match parse_message_payload(raw_message) {
        Ok(data_string) => {
            // A little bit clumsy, but due to the Deserialize trait's lifetime requirements,
            // we have to pass the data string back too.
            Ok(serde_json::from_str(&data_string).unwrap())
        }
        Err(e) => {
            Err(e)
        }
    }
}

// Maps integers to the message type enumeration
pub fn parse_message_type(data: &[u8]) -> (MessageType, &[u8]) {
    let (type_bytes, remainder) = data.split_at(size_of::<u16>());
    let message_type = match u16::from_be_bytes(type_bytes.try_into().unwrap()) {
        0 => MessageType::Unsupported,
        1 => MessageType::ProtocolError,
        2 => MessageType::ConnectRequest,
        3 => MessageType::ConnectResponse,
        4 => MessageType::DisconnectRequest,
        5 => MessageType::DisconnectResponse,
        6 => MessageType::SupportedGamesRequest,
        7 => MessageType::SupportedGamesResponse,
        8 => MessageType::LobbyListRequest,
        9 => MessageType::LobbyListResponse,
        10 => MessageType::CreateLobbyRequest,
        11 => MessageType::JoinLobbyRequest,
        12 => MessageType::ReturnToLobbyRequest,
        13 => MessageType::LobbyInfoRequest,
        14 => MessageType::LobbyInfoResponse,
        15 => MessageType::LeaveLobbyRequest,
        16 => MessageType::LeaveLobbyResponse,
        17 => MessageType::StartGameRequest,
        18 => MessageType::MoveRequest,
        19 => MessageType::GameStateResponse,
        20 => MessageType::UnsolicitedMessage,
        _ => MessageType::Unsupported
    };
    (message_type, remainder)
}

// Map integers to status code enumeration
pub fn parse_status_code(data: &[u8]) -> (StatusCode, &[u8]) {
    let (status_bytes, remainder) = data.split_at(size_of::<u16>());
    let status_type = match u16::from_be_bytes(status_bytes.try_into().unwrap()) {
        0 => StatusCode::UnexpectedError,
        1 => StatusCode::Success,
        2 => StatusCode::DataParseError,
        3 => StatusCode::DataIntegrityError,
        4 => StatusCode::MessageSequenceError,
        5 => StatusCode::MalformedBody,
        6 => StatusCode::UnsupportedRequestType,
        7 => StatusCode::UnsupportedAuthMethod,
        8 => StatusCode::UnsupportedGame,
        9 => StatusCode::NoActiveSession,
        10 => StatusCode::LobbyFull,
        11 => StatusCode::AlreadyInALobby,
        12 => StatusCode::GameStarted,
        13 => StatusCode::NotInLobby,
        14 => StatusCode::GameSessionNotFound,
        15 => StatusCode::LobbyNotFound,
        16 => StatusCode::GameStartCriteriaNotMet,
        17 => StatusCode::GameOver,
        18 => StatusCode::InvalidMove,
        _ => StatusCode::UnexpectedError
    };
    (status_type, remainder)
}