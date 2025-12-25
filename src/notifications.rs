use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

/// The type/level of the notification, determining its styling and semantic meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum NotificationType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Data structure representing a single toast notification.
#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub type_: NotificationType,
    pub timer: Timer,
    pub id: u64,
}

/// The buffer resource holding all active notifications.
#[derive(Resource, Default)]
pub struct NotificationQueue {
    pub messages: Vec<Notification>,
    pub next_id: u64,
}

/// The event used to trigger a new notification.
#[derive(Event)]
pub struct Notify {
    pub message: String,
    pub type_: NotificationType,
}

impl Notify {
    pub fn new(type_: NotificationType, msg: impl Into<String>) -> Self {
        Self {
            type_,
            message: msg.into(),
        }
    }
    
    pub fn info(msg: impl Into<String>) -> Self {
        Self::new(NotificationType::Info, msg)
    }

    pub fn success(msg: impl Into<String>) -> Self {
        Self::new(NotificationType::Success, msg)
    }

    pub fn warning(msg: impl Into<String>) -> Self {
        Self::new(NotificationType::Warning, msg)
    }

    pub fn error(msg: impl Into<String>) -> Self {
        Self::new(NotificationType::Error, msg)
    }
}

/// Observer to handle `Notify` events and add them to the queue.
pub fn on_notify(trigger: On<Notify>, mut queue: ResMut<NotificationQueue>) {
    let event = trigger.event();
    let msg = event.message.clone();
    let type_ = event.type_;

    // Log based on type
    match type_ {
        NotificationType::Error => error!("Notification: {}", msg),
        NotificationType::Warning => warn!("Notification: {}", msg),
        _ => info!("Notification: {}", msg),
    }
    
    let id = queue.next_id;
    queue.next_id += 1;
    
    queue.messages.push(Notification {
        message: msg,
        type_,
        timer: Timer::from_seconds(5.0, TimerMode::Once), // 5s default for standard toasts
        id,
    });
}

/// System to tick timers and remove expired notifications.
pub fn notification_lifecycle(time: Res<Time>, mut queue: ResMut<NotificationQueue>) {
    for note in &mut queue.messages {
        note.timer.tick(time.delta());
    }
    queue.messages.retain(|n| !n.timer.is_finished());
}

/// System to visualize notifications using egui (Temporary UI).
pub fn ui_notification_system(mut egui: EguiContexts, mut queue: ResMut<NotificationQueue>) {
    let Ok(ctx) = egui.ctx_mut() else {
        return;
    };
    
    egui::Window::new("Notifications")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-10.0, -10.0))
        .resizable(false)
        .title_bar(false)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
             let mut to_remove = Vec::new();

             // Iterate to show older messages first (stacking up) or newest first?
             // Usually toasts stack up from bottom. 
             // If we want the newest at the bottom (standard for bottom-anchor), we just iterate normally.
             for note in &mut queue.messages {
                 let (bg_color, stroke_color) = match note.type_ {
                     NotificationType::Info => (egui::Color32::from_rgb(20, 20, 50), egui::Color32::LIGHT_BLUE),
                     NotificationType::Success => (egui::Color32::from_rgb(20, 50, 20), egui::Color32::GREEN),
                     NotificationType::Warning => (egui::Color32::from_rgb(50, 50, 20), egui::Color32::YELLOW),
                     NotificationType::Error => (egui::Color32::from_rgb(50, 20, 20), egui::Color32::RED),
                 };

                 egui::Frame::window(ui.style())
                    .fill(bg_color)
                    .stroke(egui::Stroke::new(1.0, stroke_color))
                    .inner_margin(10.0)
                    .corner_radius(5.0)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            // Optional Icon
                            let icon = match note.type_ {
                                NotificationType::Info => "ℹ",
                                NotificationType::Success => "✔",
                                NotificationType::Warning => "⚠",
                                NotificationType::Error => "❗",
                            };
                            ui.label(icon);
                            
                            ui.label(
                                egui::RichText::new(&note.message)
                                .color(egui::Color32::WHITE)
                                .strong()
                            );
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.small_button("✖").clicked() {
                                    to_remove.push(note.id);
                                }
                            });
                        });
                    });
                 ui.add_space(5.0);
             }

             if !to_remove.is_empty() {
                 queue.messages.retain(|n| !to_remove.contains(&n.id));
             }
        });
}
