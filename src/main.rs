use std::io::Write;
use std::net::TcpStream;
use std::thread;
use eframe::egui;
use crate::protocol::{ConnectionStatus, GameProtocolClient};

mod protocol;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eframe::run_native(
        "Game protocol client",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(GameClient::default())),
    );
}

struct GameClient {
    name: String,
    age: u32,
    protocol_handler: GameProtocolClient
}

impl Default for GameClient {
    fn default() -> Self {
        Self {
            name: "Arthur".to_owned(),
            age: 42,
            protocol_handler: GameProtocolClient::new("127.0.0.1", "7878")
        }
    }
}

impl eframe::App for GameClient {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My egui Application");
            ui.horizontal(|ui| {
                ui.label("Your name: ");
                ui.text_edit_singleline(&mut self.name);
            });
            ui.add(egui::Slider::new(&mut self.age, 0..=120).text("age"));
            if ui.button("Click each year").clicked() {
                self.age += 1;
            }
            ui.label(format!("Hello '{}', age {}", self.name, self.age));

            let button_text = match self.protocol_handler.get_connection_status() {
                ConnectionStatus::Disconnected => "Connect to server",
                ConnectionStatus::Connecting => "Connecting...",
                ConnectionStatus::Connected => "Connected!",
            };

            let connect_button = ui.button(button_text);
            let connect_clicked = connect_button.clicked();
            if connect_clicked {
                self.protocol_handler.connect();
            }

            if matches!(self.protocol_handler.get_connection_status(), ConnectionStatus::Connected) {
                let send_message_button = ui.button("Send message").clicked();
                if send_message_button {
                    self.protocol_handler.send_message();
                }
            }
        });
    }
}
