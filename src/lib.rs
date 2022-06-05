pub use client::GameProtocolClient;
pub use server::GameProtocolServer;
pub mod game_module;
pub mod enums;

mod client;
mod server;
mod common_message_utils;
mod shared_data;