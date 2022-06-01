use std::any::Any;
use std::io::empty;
use std::sync::Arc;
use serde_json;
use serde::{Serialize, Deserialize};
use game_protocol::game_module::{GameState, GameMove, GameModule, GameMetadata};

pub struct TicTacToe {
    players: Vec<String>,
    state: TicTacToeState,
    metadata: GameMetadata
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellElement {
    None,
    X,
    O
}

// Represents the game state.
#[derive(Serialize, Deserialize)]
pub struct TicTacToeState {
    pub board: Vec<Vec<CellElement>>,
    pub this_turn: CellElement,
    pub x_player_id: String,
    pub o_player_id: String,
    pub winner: Option<String>,
    pub game_over: bool
}

#[derive(Serialize, Deserialize)]
pub struct TicTacToeMove {
    pub board_index: (usize, usize),
    pub symbol: CellElement
}

#[typetag::serde]
impl GameState for TicTacToeState {
    fn clone(&self) -> Box<dyn GameState> {
        Box::new(TicTacToeState {
            board: self.board.clone(),
            this_turn: self.this_turn,
            x_player_id: self.x_player_id.clone(),
            o_player_id: self.o_player_id.clone(),
            winner: self.winner.clone(),
            game_over: self.game_over
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[typetag::serde]
impl GameMove for TicTacToeMove {
    fn clone(&self) -> Box<dyn GameMove> {
        Box::new(TicTacToeMove {
            board_index: self.board_index,
            symbol: self.symbol
        })
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl GameModule for TicTacToe {
    fn new() -> Self where Self: Sized {
        Self {
            players: vec![],
            state: TicTacToeState {
                board: vec![
                    vec![CellElement::None; 3],
                    vec![CellElement::None; 3],
                    vec![CellElement::None; 3],
                ],
                this_turn: CellElement::X,  // X goes first
                x_player_id: "".to_string(),
                o_player_id: "".to_string(),
                winner: None,
                game_over: false
            },
            metadata: GameMetadata {
                game_title: String::from("Tic-tac-toe"),
                version: String::from("1.0"),
                max_players: 2,
                min_required_players: 2
            }
        }
    }

    fn get_metadata(&self) -> &GameMetadata {
        &self.metadata
    }

    fn add_player(&mut self, id: String) -> bool {
        if self.players.contains(&id) || self.players.len() >= self.metadata.max_players {
            return false;
        }
        self.players.push(id.clone());

        if self.state.x_player_id.eq("") {
            self.state.x_player_id = id;
        } else if self.state.o_player_id.eq("") {
            self.state.o_player_id = id;
        }

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

    fn get_player_num(&self) -> usize {
        self.players.len()
    }

    fn get_game_state(&self) -> &dyn GameState {
        &self.state
    }

    fn set_game_state(&mut self, new_state: Box<dyn GameState>) {
        let cast_state = new_state.as_any().downcast_ref::<TicTacToeState>().unwrap();
        self.state.board = cast_state.board.clone();
        self.state.this_turn = cast_state.this_turn;
        self.state.x_player_id = cast_state.x_player_id.clone();
        self.state.o_player_id = cast_state.o_player_id.clone();
    }

    fn end_condition_met(&self) -> (bool, Option<String>) {
        // End condition is when a single symbol fills in a row, column, or diagonal
        let b = &self.state.board;

        let mut winner: Option<String> = None;
        let mut ended = false;

        let top_row = b[0][0] == b[0][1] && b[0][1] == b[0][2];
        let mid_row = b[1][0] == b[1][1] && b[1][1] == b[1][2];
        let bot_row = b[2][0] == b[2][1] && b[2][1] == b[2][2];
        let left_col = b[0][0] == b[1][0] && b[1][0] == b[2][0];
        let mid_col = b[0][1] == b[1][1] && b[1][1] == b[2][1];
        let right_col = b[0][2] == b[1][2] && b[1][2] == b[2][2];
        let tl_to_br = b[0][0] == b[1][1] && b[1][1] == b[2][2]; // Top left to bottom right diagonal
        let tr_to_bl = b[0][2] == b[1][1] && b[1][1] == b[2][0]; // Top right to bottom left diagonal

        // check b[1][1] for winner
        if mid_row || mid_col || tl_to_br || tr_to_bl {
            if matches!(b[1][1], CellElement::O) {
                winner = Some(self.state.o_player_id.clone());
                ended = true;
            } else if matches!(b[1][1], CellElement::X) {
                winner = Some(self.state.x_player_id.clone());
                ended = true;
            }
        }

        // check b[0][2]
        if top_row || right_col {
            if matches!(b[0][2], CellElement::O) {
                winner = Some(self.state.o_player_id.clone());
                ended = true;
            } else if matches!(b[0][2], CellElement::X) {
                winner = Some(self.state.x_player_id.clone());
                ended = true;
            }
        }

        // check b[2][0]
        if bot_row || left_col {
            if matches!(b[2][0], CellElement::O) {
                winner = Some(self.state.o_player_id.clone());
                ended = true;
            } else if matches!(b[2][0], CellElement::X) {
                winner = Some(self.state.x_player_id.clone());
                ended = true;
            }
        }

        // Check if board is full
        let mut is_full = true;
        for row in b.iter() {
            for cell in row.iter() {
                if matches!(cell, CellElement::None) {
                    is_full = false;
                    break;
                }
            }
        }

        // Game is over if there is no winner and if the board is full
        if winner.is_none() && is_full {
            ended = true;
        }
        (ended, winner)
    }

    fn is_valid_move(&self, move_to_test: &Box<dyn GameMove>) -> bool {
        let cast_move = move_to_test.as_any().downcast_ref::<TicTacToeMove>().unwrap();
        let correct_turn = cast_move.symbol == self.state.this_turn;
        let empty_space = matches!(self.state.board[cast_move.board_index.0][cast_move.board_index.1], CellElement::None);
        correct_turn && empty_space
    }

    fn apply_move(&mut self, move_to_apply: &Box<dyn GameMove>) {
        let cast_move = move_to_apply.as_any().downcast_ref::<TicTacToeMove>().unwrap();

        // Change the turn to the other player
        match cast_move.symbol {
            CellElement::None => {}
            CellElement::X => {
                self.state.this_turn = CellElement::O;
            }
            CellElement::O => {
                self.state.this_turn = CellElement::X;
            }
        }

        self.state.board[cast_move.board_index.0][cast_move.board_index.1] = cast_move.symbol;
    }

    fn init_new(&self) -> Box<dyn GameModule> {
        Box::new(Self::new())
    }
}