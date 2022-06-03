use std::any::Any;
use std::fmt::format;
use std::rc::Rc;
use std::sync::Arc;
use serde::{Serialize, Deserialize};

// A trait that game_module modules supported by the protocol MUST implement.
// U is the data type representing the internal game_module state of the game_module.
// V is the data type representing the move players make in the game_module.
pub trait GameModule: Send + Sync {
    fn new() -> Self where Self: Sized; // Create a new instance of the game_module module.
    fn init_new(&self) -> Box<dyn GameModule>; // Returns new instantiation of this GameModule. Basically the same as new, but at the time of creating a game session, new is inaccessible.
    fn get_metadata(&self) -> &GameMetadata; // Returns a copy of the game's metadata;
    fn add_player(&mut self, id: String) -> bool; // Adds a player to the game_module. Returns if the player was successfully added.
    fn remove_player(&mut self, id: String) -> bool; // Removes a player from the game_module. Returns if the player was successfully removed.
    fn get_player_num(&self) -> usize; // Returns number of players in game.
    fn get_game_state(&self) -> &dyn GameState; // Returns a copy of the internal game state.
    fn set_game_state(&mut self, new_state: Box<dyn GameState>); // Set the game state to whatever is passed in. You will have to cast the trait object into the implementing type
    fn end_condition_met(&self) -> (bool, Option<String>); // Has the game reached a termination state yet? Return if the game has ended, and the ID of the player that one if applicable to the game
    fn is_valid_move(&self, move_to_test: &Box<dyn GameMove>) -> bool; // Checks if a move is valid or returns a valid game_module state after applying it.
    fn apply_move(&mut self, move_to_apply: &Box<dyn GameMove>); // Applies a given move to the game_module state. Must check if move is valid before applying.
}

#[typetag::serde]
pub trait GameState {
    fn clone(&self) -> Box<dyn GameState>;
    fn as_any(&self) -> &dyn Any;
}

#[typetag::serde]
pub trait GameMove {
    fn clone(&self) -> Box<dyn GameMove>;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GameMetadata {
    pub game_title: String,
    pub version: String,
    pub max_players: usize,
    pub min_required_players: usize
}

impl GameMetadata {
    pub fn get_game_type_id(&self) -> String {
        format!("{} v{}", self.game_title, self.version)
    }
}
