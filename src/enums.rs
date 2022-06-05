/*
    Creates enums for messages, status, and protocol state
 */

// For types of messages
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MessageType {
    Unsupported,
    ProtocolError,
    ConnectRequest,
    ConnectResponse,
    DisconnectRequest,
    DisconnectResponse,
    SupportedGamesRequest,
    SupportedGamesResponse,
    LobbyListRequest,
    LobbyListResponse,
    CreateLobbyRequest,
    JoinLobbyRequest,
    ReturnToLobbyRequest,
    LobbyInfoRequest,
    LobbyInfoResponse,
    LeaveLobbyRequest,
    LeaveLobbyResponse,
    StartGameRequest,
    MoveRequest,
    GameStateResponse,
    MissingMessageResponse,
    UnsolicitedMessage
}

// For server message status
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StatusCode {
    UnexpectedError,
    Success,
    DataParseError, // TODO problem with getting data from byte vec?
    DataIntegrityError,
    MessageSequenceError,
    MalformedBody, // TODO catch errors when deserializing
    UnsupportedRequestType,
    UnsupportedAuthMethod,
    UnsupportedGame,
    NoActiveSession,
    LobbyFull,
    AlreadyInALobby,
    GameStarted,
    NotInLobby,
    GameSessionNotFound,
    LobbyNotFound,
    GameStartCriteriaNotMet,
    GameOver,
    InvalidMove
}

// For general game_protocol state
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ProtocolState {
    Closed,
    Authenticating,
    Idle,
    ClosingConnection,
    GettingLobbies,
    GettingSupportedGames,
    CreatingLobby,
    JoiningLobby,
    InLobby,
    LeavingLobby,
    GettingLobbyInfo,
    CreatingGameSession,
    GameRunning,
    GettingGameState,
    LeavingGameSession
}

// For errors to be used with Result return types
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ServerError {
    ChecksumError,
    BodySizeError,
    BytesToStringError,
    DeserializeError
}