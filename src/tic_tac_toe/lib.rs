use game_protocol::game_module::{GameState, GameMove, GameModule};

pub struct TicTacToe {
    game_title: String,
    version: String,
    players: Vec<String>,
    max_players: u8
}

pub struct TicTacToeState {

}

pub struct TicTacToeMove {

}

impl GameState for TicTacToeState {

}

impl GameMove for TicTacToeMove {

}

impl GameModule for TicTacToe {
    fn new() -> Self where Self: Sized {
        Self {
            game_title: String::from("Tic-tac-toe"),
            version: String::from("1.0"),
            players: vec![],
            max_players: 2
        }
    }

    fn to_string(&self) -> String {
        format!("{{game_title: {}, max_players: {}}}", self.game_title, self.max_players)
    }

    fn get_version(&self) -> String {
        self.version.clone()
    }

    fn start_criteria_met(&self) -> bool {
        todo!()
    }

    fn start(&mut self) -> bool {
        todo!()
    }

    fn get_game_title(&self) -> String {
        self.game_title.clone()
    }

    fn add_player(&mut self, id: String) -> bool {
        if self.players.contains(&id) || self.players.len() >= self.max_players as usize {
            return false;
        }
        self.players.push(id);
        return true;
    }

    fn remove_player(&mut self, id: String) -> bool {
        for i in 0..self.players.len() {
            if self.players[i].eq(&id) {
                self.players.remove(i);
                return true;
            }
        }
        return false;
    }

    fn get_max_players(&self) -> u8 {
        self.max_players
    }

    fn get_game_state(&self) -> Box<dyn GameState> {
        todo!()
    }

    fn is_valid_move(&self, move_to_test: Box<dyn GameMove>) -> bool {
        todo!()
    }

    fn apply_move(&mut self, move_to_apply: Box<dyn GameMove>) {
        todo!()
    }
}