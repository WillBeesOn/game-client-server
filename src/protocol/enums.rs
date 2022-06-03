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

// For server_bin message status
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StatusCode {
    UnexpectedError,
    Success,
    DataParseError,
    DataIntegrityError,
    MessageSequenceError,
    MalformedBody,
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

// For general protocol state
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