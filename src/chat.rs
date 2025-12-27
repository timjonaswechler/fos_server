use {
    crate::protocol::{ClientChat, ServerChat},
    bevy::prelude::*,
    bevy_egui::{egui, EguiContexts},
};

pub struct ChatPlugin;

impl Plugin for ChatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChatState>()
            .add_systems(Update, (receive_chat_messages, handle_chat_input));
    }
}

#[derive(Resource, Default)]
pub struct ChatState {
    pub messages: Vec<(String, String)>, // (Sender, Content)
    pub input: String,
    pub is_open: bool,
    pub has_focus: bool,
}

fn receive_chat_messages(
    mut chat_events: MessageReader<ServerChat>,
    mut chat_state: ResMut<ChatState>,
) {
    for event in chat_events.read() {
        chat_state
            .messages
            .push((event.sender.clone(), event.text.clone()));

        // Keep history limited (optional, e.g., last 100 messages)
        if chat_state.messages.len() > 100 {
            chat_state.messages.remove(0);
        }
    }
}

fn handle_chat_input(
    mut chat_state: ResMut<ChatState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut client_chat_writer: MessageWriter<ClientChat>,
) {
    // Open chat with Enter or T if not open
    if !chat_state.is_open {
        if keyboard.just_pressed(KeyCode::Enter) || keyboard.just_pressed(KeyCode::KeyT) {
            chat_state.is_open = true;
            chat_state.has_focus = true;
        }
        return;
    }

    // Close with Escape
    if keyboard.just_pressed(KeyCode::Escape) {
        chat_state.is_open = false;
        chat_state.has_focus = false;
        return;
    }

    // Send with Enter
    if keyboard.just_pressed(KeyCode::Enter) && chat_state.has_focus {
        if !chat_state.input.trim().is_empty() {
            client_chat_writer.write(ClientChat {
                text: chat_state.input.clone(),
            });
            chat_state.input.clear();
        }
        // Keep focus or close after send? Usually keep focus in modern games,
        // but often close in simple MMOs. Let's keep it open for now,
        // or user can press Esc to close.
        // For now: clear input and keep focus.
    }
}

pub fn render_chat_ui(mut egui: EguiContexts, mut chat_state: ResMut<ChatState>) {
    // Always show chat history (maybe faded?) or only when open?
    // Let's emulate a typical MMO chat: Always visible background (transparent),
    // input only when "open".

    let Ok(ctx) = egui.ctx_mut() else {
        return;
    };

    // Define the window style
    let window_frame = egui::Frame::window(&ctx.style())
        .fill(egui::Color32::from_rgba_premultiplied(0, 50, 0, 150)) // Semi-transparent black
        .stroke(egui::Stroke::NONE)
        .inner_margin(5.0);

    egui::Window::new("Chat")
        .frame(window_frame)
        .anchor(egui::Align2::LEFT_BOTTOM, egui::Vec2::new(10.0, -10.0))
        .title_bar(false)
        .resizable(false)
        .fixed_size(egui::Vec2::new(300.0, 200.0))
        .show(ctx, |ui| {
            ui.vertical(|ui| {
                // Chat History Area
                let scroll_area = egui::ScrollArea::vertical()
                    .stick_to_bottom(true)
                    .max_height(160.0); // Leave space for input

                scroll_area.show(ui, |ui| {
                    ui.set_min_width(ui.available_width());

                    for (sender, text) in &chat_state.messages {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("{}:", sender))
                                    .color(egui::Color32::LIGHT_BLUE)
                                    .strong(),
                            );
                            ui.label(egui::RichText::new(text).color(egui::Color32::WHITE));
                        });
                    }
                });

                // Input Area (only if open)
                if chat_state.is_open {
                    ui.separator();
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut chat_state.input)
                            .hint_text("Press Enter to send...")
                            .desired_width(f32::INFINITY)
                            .lock_focus(true), // Keep focus
                    );

                    if chat_state.has_focus {
                        response.request_focus();
                    }
                } else {
                    // Show a hint how to open chat if it's not empty?
                    // Or just invisible.
                    // ui.label(egui::RichText::new("Press [Enter] to chat").weak().small());
                }
            });
        });
}
