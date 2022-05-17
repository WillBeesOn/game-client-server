use std::rc::Rc;
use serde::{Serialize, Deserialize};

// A trait that game_module modules supported by the protocol MUST implement.
// U is the data type representing the internal game_module state of the game_module.
// V is the data type representing the move players make in the game_module.
pub trait GameModule: Send + Sync {
    fn new() -> Self where Self: Sized; // Create a new instance of the game_module module.
    fn to_string(&self) -> String; // A String representation of the game module. Needed as a workaround because server_bin and client_bin need to store a Vec<dyn GameModule> to support multiple modules. But GameModule cannot implement Serialize for serde_json because of this.
    fn get_version(&self) -> String; // Return the version of the game module
    fn start_criteria_met(&self) -> bool; // Checks if the game_module's start criteria have been met.
    fn start(&mut self) -> bool; // Attempts to start the game_module. Returns if the game_module has been successfully started.
    fn get_game_title(&self) -> String; // Get a string representing the name of the game_module
    fn add_player(&mut self, id: String) -> bool; // Adds a player to the game_module. Returns if the player was successfully added.
    fn remove_player(&mut self, id: String) -> bool; // Removes a player from the game_module. Returns if the player was successfully removed.
    fn get_max_players(&self) -> u8; // Returns the maximum amount of players a game_module can have.
    fn get_game_state(&self) -> Box<dyn GameState>; // Returns the internal game_module state at the time of calling this function.
    fn is_valid_move(&self, move_to_test: Box<dyn GameMove>) -> bool; // Checks if a move is valid or returns a valid game_module state after applying it.
    fn apply_move(&mut self, move_to_apply: Box<dyn GameMove>); // Applies a given move to the game_module state. Must check if move is valid before applying.
}

pub trait GameState { }
pub trait GameMove { }

#[derive(Serialize, Deserialize, Clone)]
pub struct GameMetadata {
    pub game_title: String,
    pub version: String,
    pub initialized_checksum: u32
}

impl GameMetadata {
    pub fn from(game_module: Rc<dyn GameModule>) -> Self {
        Self {
            game_title: game_module.get_game_title(),
            version: game_module.get_version(),
            initialized_checksum: crc32fast::hash(game_module.to_string().as_bytes())
        }
    }

    pub fn to_string(&self) -> String {
        format!("{} {} {}", self.game_title, self.version, self.initialized_checksum)
    }
}
