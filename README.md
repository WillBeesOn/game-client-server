# game-client-server

A library for creating a client and server programs that communicate with each other to facilitate playing simple turned based games like TicTacToe or Chess.

Also provides traits and data types for creating your own game module for the server and client to support.

See an exaple client, server, and game module implementation [here](https://github.com/WillBeesOn/game-client-server/tree/main/example). The server is quite simple but the client is a bit more complicated with a GUI implemented with the `egui` crate.

## [`game_protocol::GameProtocolClient`](https://github.com/WillBeesOn/game-client-server/tree/main/src/client/mod.rs)

Use to create a client program. 

`new() -> Self` - Creates a new instance of the client handler.

`register_game<T: 'static + GameModule>(&self)` - Registers a game module for the client to support.

`connect(&mut self, ip: &String, port: &String)` - Attempts to connect to a server at the provided socket address and establish a session.

`on_message_received_callback(&self, callback: impl Fn() + Send + Sync + 'static)` - Set a callback that is run whenever the client receives a message from the server.

`async_listen(&self)` - Listen for server messages in a separate thread.

`stop_async_listen(&self` - Stop listening for server messages asynchronously.

`disconnect(&self)` - Disconnect and destroy session with server.

`get_client_id(&self) -> String` - Returns the UUID associated with this client, which is generated server side.

`get_socket_address(&self) -> String` - Returns socket address the client is connected to.

`get_protocol_state(&self) -> ProtocolState` - Get the `ProtocolState` enum which represents the client's current network protocol state.

`request_supported_games(&self)` - Requests a list of the connected server's supported games.

`get_supported_games(&self) -> Vec<(String, String)>` - Returns a list of supported games in the form of a tuple containing the game's title and game's ID.

`request_lobby_list(&self)` - Requests a list of all the lobbies a server stores.

`get_lobby_list(&self) -> Vec<Lobby>` - Returns a list of lobbies the server is hosting.

`create_lobby(&self, game_type_id: &str)` - Request the server to create a lobby that hosts a game with the matching ID.

`join_lobby(&self, lobby_id: &str)` - Request the server to add the client to the requeted lobby.

`get_current_lobby(&self) -> Option<Lobby>` - If the client is in a lobby, return data for the lobby the client is currenly in.

`start_game(&self)` - Request the server to start the game the lobby is meant to host.

`make_move(&self, game_move: &dyn GameMove)` - Request the server perform some action in the game state.

`get_game_state(&self) -> Option<Box<dyn GameState>>` - If the client is playing a game, Get the game state of the game the client is currently playing.

`get_game_end_result(&self) -> Option<(bool, Option<String>)>` - If the client is in a game, return data about whether or not the game has ended: has the game ended, and the ID of the winner if there is a player that has won.

`refresh_current_lobby(&self)` - Request updated information for the lobby the client is in if they are in a lobby.

`leave_lobby(&self)` - Request the server to remove the client from the lobby it is in if they are in a lobby.

`return_to_lobby(&self)` - Request the server to return the client from a game session back to the lobby that hosted the game session.

## [`game_protocol:GameProtocolServer`](https://github.com/WillBeesOn/game-client-server/tree/main/src/server/mod.rs)

Use to create a server program. Getting a server started is much more straightforward and requires very little set up.

`new(ip: &str, port: &str) -> Self` - Creates a new server instance on a given socket address.

`register_game::<T: 'static + GameModule>(&self)` - Registers a game module for the server to support.

`start()` - Bind server to a TCPListener on the supplied socket address and listens for client connections.

## [`game_protocol:ProtocolState`](https://github.com/WillBeesOn/game-client-server/tree/main/src/enums.rs)
An enum used to represent the network protocol state the client is in.

## [`game_protocol::game_module`](https://github.com/WillBeesOn/game-client-server/tree/main/src/game_module/mod.rs)
A set of traits and data types which custom game modules must implement in order to be compatible with client-server operations.

`trait GameModule: Send + Sync`:
- `new() -> Self where Self: Sized` - Create a new instance of the `GameModule` object.
- `init_new(&self) -> Box<dyn GameModule>` - A factory function to create a new instance of this `GameModule` from an existing instance.
- `get_metadata(&self) -> &GameMetadata` - Returns an immutable reference to the game's metadata;
- `add_player(&mut self, id: String)` - Adds a player to the `GameModule`.
- `remove_player(&mut self, id: String)` - Removes a player from the `GameModule`.
- `get_player_num(&self) -> usize` - Returns number of players in game.
- `get_game_state(&self) -> &dyn GameState` - Returns an immutable reference of the internal game state.
- `set_game_state(&mut self, new_state: Box<dyn GameState>)` - Set the game state to whatever is passed in. You will have to downcast the trait object into the implementing type
- `end_condition_met(&self) -> (bool, Option<String>)` - Has the game reached a termination state yet? Return whether or not the game has ended, and the ID of the player that one if applicable to the game
- `is_valid_move(&self, move_to_test: &Box<dyn GameMove>) -> bool` - Checks if a `GameMove` is valid in the current game state.
- `apply_move(&mut self, move_to_apply: &Box<dyn GameMove>)` - Applies a given move to the `GameState`. Not necessary to check if move is valid in this function, but it is recommended.

`trait GameState` and `trait GameMove`
- Implementing types must derive `serde::Serialize` and `serde::Deserialize`.
- Implementing types must use the `#[typetag::serde]` macro provided by [typetag](https://crates.io/crates/typetag)
- `clone(&self) -> Box<dyn GameState>` - Create a clone of the curent `GameState` object
- `as_any(&self) -> &dyn Any` - Cast into `Any` type. Typically you just need to return `self` to implement this.
