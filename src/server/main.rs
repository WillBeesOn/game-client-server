use game_protocol::server::GameProtocolServer;
use tic_tac_toe::TicTacToe;

/*
    Simple program that uses the game protocol's server and registers TicTacToe as a playable game.
    Server module is robust enough that not much needs to be done to start it up.
 */

fn main() {
    let mut server = GameProtocolServer::new("localhost", "7878");
    server.register_game::<TicTacToe>();
    server.start();
}
