use eframe::egui;
use egui::Button;
use game_protocol::client::GameProtocolClient;
use game_protocol::enums::ProtocolState;
use tic_tac_toe::{CellElement, TicTacToe, TicTacToeMove, TicTacToeState};

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
    got_initial_lobbies: bool,
    initialized_on_message_received: bool,
    is_listening_async: bool,
    ip: String,
    port: String,
}

impl Default for GameClient {
    fn default() -> Self {
        let mut protocol = GameProtocolClient::new();
        protocol.register_game::<TicTacToe>();
        Self {
            protocol_handler: protocol,
            got_initial_lobbies: false,
            initialized_on_message_received: false,
            is_listening_async: false,
            ip: String::from("127.0.0.1"),
            port: String::from("7878"),
        }
    }
}

impl eframe::App for GameClient {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        // Set GUI to repaint whenever a protocol message is received.
        if !self.initialized_on_message_received {
            let ctx_clone = ctx.clone();
            self.protocol_handler.on_message_received_callback(move || {
                ctx_clone.request_repaint();
            });
            self.initialized_on_message_received = true;
        }

        // Render the GUI
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
                    ui.label(format!("Connected to server at {}", self.protocol_handler.get_socket_address()));
                });
            }

            // What to do in the GUI while client_bin is not in a lobby or game session.
            // The client_bin can only request a list of lobbies or attempt to create/join a lobby.
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
                    self.protocol_handler.stop_async_listen();
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
                                if ui.button(format!("Create lobby for {}", &game.0)).clicked() {
                                    self.protocol_handler.create_lobby(&game.1);
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
                    // TODO make sure client can only join games that it supports
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for l in lobbies.iter() {
                            ui.horizontal(|ui| {
                                if ui.button("Join Lobby").clicked() {
                                    self.protocol_handler.join_lobby(&l.id)
                                };
                                ui.label(format!("Game: {}. Players: {}/{}. Started: {}", l.game_metadata.game_title, l.player_ids.len(), l.game_metadata.max_players, l.game_started));
                            });
                        }
                    });

                } else {
                    ui.label("No lobbies available.");
                }
            }

            // UI to show while in the lobby.
            if matches!(connection_status, ProtocolState::InLobby) {
                if !self.is_listening_async {
                    self.protocol_handler.async_listen();
                }

                if ui.button("Leave Lobby").clicked() {
                    self.protocol_handler.leave_lobby();
                }

                // Display current lobby info
                if let Some(lobby) = self.protocol_handler.get_current_lobby() {
                    if self.protocol_handler.get_client_id() == lobby.owner {
                        let enable = lobby.player_ids.len() >= lobby.game_metadata.min_required_players;
                        if ui.add_enabled(enable, Button::new("Start game")).clicked() {
                            self.protocol_handler.start_game(&lobby.id);
                        }
                    }
                    ui.label(format!("Players: {}/{}", lobby.player_ids.len(), lobby.game_metadata.max_players));
                }
            }

            // UI to show while the game has started.
            if matches!(connection_status, ProtocolState::GameRunning) {

                // If there is a game state stored by the protocol client, then get it and display it.
                if let Some(game_state) = self.protocol_handler.get_game_state() {
                    let game_type_id = self.protocol_handler.get_current_lobby().unwrap().game_metadata.get_game_type_id();

                    // Only supports Tic-tac-toe v1.0 for now. Render its UI and handle the game logic.
                    if game_type_id.eq("Tic-tac-toe v1.0") {
                        let my_id = self.protocol_handler.get_client_id();
                        let mut handle_user_input = true;
                        if let Some(result) = self.protocol_handler.get_game_end_result() {
                            if result.0 {
                                handle_user_input = false;
                                 if let Some(winner) = result.1 {
                                     if winner.eq(&my_id) {
                                         ui.label("Game over, you win!");

                                     } else {
                                         ui.label("Game over, you lose!");
                                     }
                                 } else {
                                     ui.label("Game over, it's a draw!");
                                 }
                                if ui.button("Return to lobby.").clicked() {
                                    self.protocol_handler.return_to_lobby();
                                }
                            }
                        }

                        let state = game_state.as_any().downcast_ref::<TicTacToeState>().unwrap();
                        let mut clicked_space: Option<(usize, usize)> = None;
                        for (i, row) in state.board.iter().enumerate() {
                            ui.horizontal(|ui| {
                               for (j, cell) in row.iter().enumerate() {
                                   let button_label = match cell {
                                       CellElement::None => "   ",
                                       CellElement::X => "X",
                                       CellElement::O => "O"
                                   };
                                   if ui.button(button_label).clicked() && clicked_space.is_none() {
                                       clicked_space = Some((i, j));
                                   }
                               }
                            });
                        }

                        if handle_user_input {
                            // If the user clicked a button in the board, check if it's a valid move and if so send it to server.
                            if let Some(click_data) = clicked_space {
                                let mut symbol = CellElement::None;
                                if state.x_player_id.eq(&my_id) {
                                    symbol = CellElement::X;
                                } else if state.o_player_id.eq(&my_id) {
                                    symbol = CellElement::O;
                                }

                                let move_obj = TicTacToeMove {
                                    board_index: (click_data.0, click_data.1),
                                    symbol
                                };
                                self.protocol_handler.make_move(&move_obj);
                            }
                        }
                    }
                }

            }
        });
    }
}
