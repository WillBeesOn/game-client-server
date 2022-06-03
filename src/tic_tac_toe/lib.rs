use std::any::Any;
use serde::{Serialize, Deserialize};
use game_protocol::game_module::{GameState, GameMove, GameModule, GameMetadata};

/*
    This is an implementation of a game module that can be registered and played using the game_protocol.
    This implements the traits provided by game_protocol::game_module in order to create a game module that is
    compatible with protocol operations.
 */

// Game module struct data
pub struct TicTacToe {
    players: Vec<String>,
    state: TicTacToeState,
    metadata: GameMetadata
}

// Enumeration of values to show in cells of a tic-tac-toe board.
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellElement {
    None,
    X,
    O
}

// Represents the tic-tac-toe game state.
#[derive(Serialize, Deserialize)]
pub struct TicTacToeState {
    pub board: Vec<Vec<CellElement>>,
    pub this_turn: CellElement,
    pub x_player_id: String,
    pub o_player_id: String,
    pub winner: Option<String>,
    pub game_over: bool
}

// Represents a move a player can make in the game state
#[derive(Serialize, Deserialize)]
pub struct TicTacToeMove {
    pub board_index: (usize, usize),
    pub symbol: CellElement
}

// Implementing required GameState functions
#[typetag::serde]
impl GameState for TicTacToeState {
    // Creates a clone of this instance of the game state. Put it into a Box pointer to satisfy
    // un-sizable nature of trait objects
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

    // Used to cast trait object into an Any type which
    // can then be downcast into the specific TicTacToeState type
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Implementing required GameMove functions
#[typetag::serde]
impl GameMove for TicTacToeMove {
    // Creates a clone of this instance of the game state. Put it into a Box pointer to satisfy
    // un-sizable nature of trait objects
    fn clone(&self) -> Box<dyn GameMove> {
        Box::new(TicTacToeMove {
            board_index: self.board_index,
            symbol: self.symbol
        })
    }

    // Used to cast trait object into an Any type which
    // can then be downcast into the specific TicTacToeMove type
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// Implement required GameModule functions for TicTacToe game module
impl GameModule for TicTacToe {
    // Constructor to create new instance of TicTacToe module.
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
                game_title: "Tic-tac-toe".to_string(),
                version: "1.0".to_string(),
                max_players: 2,
                min_required_players: 2
            }
        }
    }

    // A factory function to create a new, fresh instance of TicTacToe game module from an existing instance.
    fn init_new(&self) -> Box<dyn GameModule> {
        Box::new(Self::new())
    }

    // Return an immutable reference to the module's metadata
    fn get_metadata(&self) -> &GameMetadata {
        &self.metadata
    }

    // Add a player to the game session.
    fn add_player(&mut self, id: String) {
        // Don't add any players if the player is already in the session and the amount of players has already reached the maximum
        if self.players.contains(&id) || self.players.len() >= self.metadata.max_players {
            return;
        }
        self.players.push(id.clone());

        // Set which player is the X and O player.
        if self.state.x_player_id.eq("") {
            self.state.x_player_id = id;
        } else if self.state.o_player_id.eq("") {
            self.state.o_player_id = id;
        }

        println!("ADd len: {}", self.players.len());
    }

    // Removes a player from the game.
    fn remove_player(&mut self, id: String) {
        // Iterate through player IDs. Must do it by index since removal from vectors is index based.
        for i in 0..self.players.len() {
            if self.players[i].eq(&id) {
                self.players.remove(i);

                // Clear the X or O player if the player to be removed is either of them.
                if self.state.x_player_id.eq(&id) {
                    self.state.x_player_id = "".to_string();
                } else if self.state.o_player_id.eq(&id) {
                    self.state.o_player_id = "".to_string();
                }
                break;
            }
        }
    }

    // Gets the number of players in the game session
    fn get_player_num(&self) -> usize {
        self.players.len()
    }

    // Get an immutable reference to the game state
    fn get_game_state(&self) -> &dyn GameState {
        &self.state
    }

    // Given a game state object, cast it to a TicTacToeState object and apply the data within to this
    // module instance's state
    fn set_game_state(&mut self, new_state: Box<dyn GameState>) {
        let cast_state = new_state.as_any().downcast_ref::<TicTacToeState>().unwrap();
        self.state.board = cast_state.board.clone();
        self.state.this_turn = cast_state.this_turn;
        self.state.x_player_id = cast_state.x_player_id.clone();
        self.state.o_player_id = cast_state.o_player_id.clone();
    }

    // Check if the tic-tac-toe game has reached a termination point.
    fn end_condition_met(&self) -> (bool, Option<String>) {
        // End condition is when a single symbol fills in a row, column, or diagonal.
        // Or if the entire board is filled and no player is the winner.
        let b = &self.state.board; // Easier to reference b than self.state.board all the time

        let mut winner: Option<String> = None;
        let mut ended = false;

        // Check if any of the rows, columns, or diagonals have the same symbol.
        let top_row = b[0][0] == b[0][1] && b[0][1] == b[0][2];
        let mid_row = b[1][0] == b[1][1] && b[1][1] == b[1][2];
        let bot_row = b[2][0] == b[2][1] && b[2][1] == b[2][2];
        let left_col = b[0][0] == b[1][0] && b[1][0] == b[2][0];
        let mid_col = b[0][1] == b[1][1] && b[1][1] == b[2][1];
        let right_col = b[0][2] == b[1][2] && b[1][2] == b[2][2];
        let tl_to_br = b[0][0] == b[1][1] && b[1][1] == b[2][2]; // Top left to bottom right diagonal
        let tr_to_bl = b[0][2] == b[1][1] && b[1][1] == b[2][0]; // Top right to bottom left diagonal

        // check b[1][1] for winner since it's a mutual cell between these rows, columns, and diagonals
        if mid_row || mid_col || tl_to_br || tr_to_bl {
            if matches!(b[1][1], CellElement::O) {
                winner = Some(self.state.o_player_id.clone());
                ended = true;
            } else if matches!(b[1][1], CellElement::X) {
                winner = Some(self.state.x_player_id.clone());
                ended = true;
            }
        }

        // check b[0][2] for winner since it's a mutual cell between this row and column
        if top_row || right_col {
            if matches!(b[0][2], CellElement::O) {
                winner = Some(self.state.o_player_id.clone());
                ended = true;
            } else if matches!(b[0][2], CellElement::X) {
                winner = Some(self.state.x_player_id.clone());
                ended = true;
            }
        }

        // check b[2][0] for winner since it's a mutual cell between this row and column
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

        // Game ends in a draw if there is no winner and if the board is full
        if winner.is_none() && is_full {
            ended = true;
        }
        (ended, winner)
    }

    // Checks if a given move is valid. Must cast GameMove trait object into TicTacToeMove
    fn is_valid_move(&self, move_to_test: &Box<dyn GameMove>) -> bool {
        let cast_move = move_to_test.as_any().downcast_ref::<TicTacToeMove>().unwrap();
        let correct_turn = cast_move.symbol == self.state.this_turn; // Requested symbol must match the symbol of the upcoming turn
        let empty_space = matches!(self.state.board[cast_move.board_index.0][cast_move.board_index.1], CellElement::None); // Moves can only be made in empty spaces.
        correct_turn && empty_space
    }

    // Apply the move to the game state.
    fn apply_move(&mut self, move_to_apply: &Box<dyn GameMove>) {

        // Check if the move is valid first
        if !self.is_valid_move(move_to_apply) {
            return;
        }

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

        // Apply the requested symbol to the requested space
        self.state.board[cast_move.board_index.0][cast_move.board_index.1] = cast_move.symbol;
    }
}