use std::any::Any;
use serde::{Serialize, Deserialize};


/*
    This module contains traits and implementations for structs that are required to be compatible with protocol operations.
    Game modules MUST implement these traits in their own data types.
 */

// Trait for the overall game module. Must be thread safe.
pub trait GameModule: Send + Sync {
    fn new() -> Self where Self: Sized; // Create a new instance of the game_module object.
    fn init_new(&self) -> Box<dyn GameModule>; // Returns new instantiation of this game module from an existing instance. A factory function basically.
    fn get_metadata(&self) -> &GameMetadata; // Returns an immutable reference to the game's metadata;
    fn add_player(&mut self, id: String); // Adds a player to the game_module.
    fn remove_player(&mut self, id: String); // Removes a player from the game_module.
    fn get_player_num(&self) -> usize; // Returns number of players in game.
    fn get_game_state(&self) -> &dyn GameState; // Returns an immutable reference of the internal game state.
    fn set_game_state(&mut self, new_state: Box<dyn GameState>); // Set the game state to whatever is passed in. You will have to downcast the trait object into the implementing type
    fn end_condition_met(&self) -> (bool, Option<String>); // Has the game reached a termination state yet? Return whether or not the game has ended, and the ID of the player that one if applicable to the game
    fn is_valid_move(&self, move_to_test: &Box<dyn GameMove>) -> bool; // Checks if a move is valid or returns a valid game_module state after applying it.
    fn apply_move(&mut self, move_to_apply: &Box<dyn GameMove>); // Applies a given move to the game_module state. Must check if move is valid before applying.
}

// Functions to implement for game state objects. Also must be serializable and deserializable.
// Use typetag::serde to allow serialization and deserialization of trait objects
#[typetag::serde]
pub trait GameState {
    fn clone(&self) -> Box<dyn GameState>;
    fn as_any(&self) -> &dyn Any;
}

// Functions to implement for game move objects. Also must be serializable and deserializable.
// Use typetag::serde to allow serialization and deserialization of trait objects
#[typetag::serde]
pub trait GameMove {
    fn clone(&self) -> Box<dyn GameMove>;
    fn as_any(&self) -> &dyn Any;
}

// Struct representing game metadata.
#[derive(Serialize, Deserialize, Clone)]
pub struct GameMetadata {
    pub game_title: String,
    pub version: String,
    pub max_players: usize,
    pub min_required_players: usize
}

impl GameMetadata {
    // Create an ID for the game. Simply the name and the version. For the puprpose of this project it doesn't need to be more complex than that.
    pub fn get_game_type_id(&self) -> String {
        format!("{} v{}", self.game_title, self.version)
    }
}
