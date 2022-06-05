pub use client::GameProtocolClient;
pub use server::GameProtocolServer;
pub use enums::ProtocolState;
pub mod game_module;

mod enums;
mod client;
mod server;
mod common_message_utils;
mod shared_data;