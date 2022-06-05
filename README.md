# game-client-server

A library for creating a client and server programs that communicate with each other to facilitate playing simple turned based games like TicTacToe or Chess.

Also provides traits and data types for creating your own game module for the server and client to support.

See an exaple client, server, and game module implementation [here](https://github.com/WillBeesOn/game-client-server/tree/main/example). The server is quite simple but the client is a bit more complicated with a GUI implemented with the `egui` crate.

## [`game_module::GameProtocolClient`](https://github.com/WillBeesOn/game-client-server/tree/main/src/client/mod.rs)

Use to create a client program. 

`new() -> Self` - Creates a new instance of the client handler.

`connect(&mut self, ip: &String, port: &String)` -

`async_listen(&self)` - 

`stop_async_listen(&self` -

`disconnect(&self)` - 

`get_socket_address(&self) -> String` - 

`get_protocol_state(&self) -> ProtocolState` - 

`get_supported_games(&self) -> Vec<(String, String)>` - 

`get_lobby_list(&self) -> Vec<Lobby>` -

`get_current_lobby(&self) -> Option<Lobby>` - 

`get_client_id(&self) -> String` - 

`get_game_end_result(&self) -> Option<(bool, Option<String>)>` - 

`on_message_received_callback(&self, callback: impl Fn() + Send + Sync + 'static)` - 

`register_game<T: 'static + GameModule>(&self)` - 

`request_lobby_list(&self)` - 

`request_supported_games(&self)` -

`refresh_current_lobby(&self)` - 

`create_lobby(&self, game_type_id: &str)` - 

`join_lobby(&self, lobby_id: &str)` - 

`leave_lobby(&self)` - 

`start_game(&self)` - 

`make_move(&self, game_move: &dyn GameMove)` - 

`return_to_lobby(&self)` - 

## [`game_module::GameProtocolServer`](https://github.com/WillBeesOn/game-client-server/tree/main/src/server/mod.rs)

Use to create a server program. Getting a server started is much more straightforward and requires very little set up.

`new(ip: &str, port: &str) -> Self` - Creates a new server instance on a given socket address.

`register_game::<T: 'static + GameModule>(&self)` - Registers a game module for the server to support.

`start()` - Bind server to a TCPListener on the supplied socket address and listens for client connections.
