use std::{thread, time};
use std::sync::{Arc, Mutex};
use eframe::egui;
use egui::Button;
use game_protocol::client::GameProtocolClient;
use game_protocol::enums::ProtocolState;
use tic_tac_toe::TicTacToe;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eframe::run_native(
        "Game protocol client_bin",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(GameClient::default())),
    );
}

struct GameClient {
    protocol_handler: GameProtocolClient,
    previous_handler_state: ProtocolState,
    got_initial_lobbies: bool,
    initialized_on_state_change_handler: bool,
    ip: String,
    port: String,
}

impl Default for GameClient {
    fn default() -> Self {
        let mut protocol = GameProtocolClient::new();
        protocol.register_game::<TicTacToe>();
        Self {
            protocol_handler: protocol,
            previous_handler_state: ProtocolState::Closed,
            got_initial_lobbies: false,
            initialized_on_state_change_handler: false,
            ip: String::from("127.0.0.1"),
            port: String::from("7878"),
        }
    }
}

impl eframe::App for GameClient {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let connection_status = self.protocol_handler.get_protocol_state();

            // While no connection is established, allow user to specify port and IP to connect to.
            if matches!(connection_status, ProtocolState::Closed) {
                // TODO need to verify IP and port number
                ui.horizontal(|ui| {
                    ui.label("Server IP address");
                    ui.text_edit_singleline(&mut self.ip);
                });
                ui.horizontal(|ui| {
                    ui.label("Port number");
                    ui.text_edit_singleline(&mut self.port);
                });

                if ui.button("Connect to server").clicked() {
                    self.protocol_handler.connect(&self.ip, &self.port);
                }
            }

            // While there is a connection established, show where the client_bin is connected to.
            if !matches!(connection_status, ProtocolState::Closed) {
                ui.horizontal(|ui| {
                    ui.label(format!("Connected to server_bin at {}", self.protocol_handler.get_socket_address()));
                });
            }

            // What to do in the GUI while client_bin is not in a lobby or game session.
            // The client_bin can only request a list of lobbies or attempt to create/join a lobby.
            // TODO need a system to allow certain GUI elements when in a range of states
            if matches!(connection_status, ProtocolState::Idle) {
                // Make initial request for list of lobbies when client_bin first enters this state.
                // Afterwards it must be requested manually.
                if !self.got_initial_lobbies {
                    self.protocol_handler.request_supported_games();
                    self.protocol_handler.request_lobby_list();
                    self.got_initial_lobbies = true;
                }

                // Allow user to disconnect from server_bin only if we're in the idle state.
                if ui.button("Disconnect").clicked() {
                    self.protocol_handler.disconnect();
                    self.got_initial_lobbies = false;
                }

                // Buttons for refreshing lobby list and creating a lobby
                ui.horizontal(|ui| {
                    if ui.button("Refresh lobby list").clicked() {
                        self.protocol_handler.request_lobby_list();
                    }

                    // Show buttons for creating lobbies of supported games. Or note there are no supported games (games the client_bin and server_bin both support).
                    ui.horizontal(|ui| {
                        let supported_games = self.protocol_handler.get_supported_games();
                        if supported_games.len() > 0 {
                            for game in self.protocol_handler.get_supported_games().iter() {
                                if ui.button(format!("Create lobby for {}", game.game_title)).clicked() {
                                    self.protocol_handler.create_lobby(&game.to_string());
                                }
                            }
                        } else {
                            ui.label("No games available.");
                        }
                    });
                });

                // If there are lobbies, render them in a scrollable list so a large number of them can be displayed
                let lobbies = self.protocol_handler.get_lobby_list();
                if lobbies.len() > 0 {
                    // TODO make sure client_bin can support the game it sees
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for l in lobbies.iter() {
                            ui.horizontal(|ui| {
                                if ui.button("Join Lobby").clicked() {
                                    self.protocol_handler.join_lobby(&l.id)
                                };
                                ui.label(format!("Game: {}. Players: {}/{}", l.game_title, l.player_ids.len(), l.max_players));
                            });
                        }
                    });

                } else {
                    ui.label("No lobbies available.");
                }
            }

            // UI to show while in the lobby.
            if matches!(connection_status, ProtocolState::InLobby) {
                if ui.button("Leave Lobby").clicked() {
                    self.protocol_handler.leave_lobby();
                }

                if ui.button("Refresh lobby").clicked() {
                    self.protocol_handler.refresh_current_lobby();
                }

                // Display current lobby info
                if let Some(lobby) = self.protocol_handler.get_current_lobby() {
                    if self.protocol_handler.get_client_id() == lobby.owner {
                        if ui.add_enabled(false, Button::new("Start game")).clicked() {

                        }
                    }
                    ui.label(format!("Players: {}/{}", lobby.player_ids.len(), lobby.max_players));
                }
            }
        });
    }
}
