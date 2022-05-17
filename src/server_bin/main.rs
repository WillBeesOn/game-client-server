use game_protocol::server::GameProtocolServer;
use tic_tac_toe::TicTacToe;

fn main() {
    let mut server = GameProtocolServer::new("127.0.0.1", "7878");
    server.register_game::<TicTacToe>();
    server.start();
}
