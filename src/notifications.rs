use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

#[derive(Clone, Debug)]
pub struct Notification {
    pub message: String,
    pub timer: Timer,
    pub id: u64,
}

#[derive(Resource, Default)]
pub struct NotificationQueue {
    pub messages: Vec<Notification>,
    pub next_id: u64,
}

#[derive(Event)]
pub struct NotifyError(pub String);

impl NotifyError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

pub fn on_notify_error(trigger: On<NotifyError>, mut queue: ResMut<NotificationQueue>) {
    let msg = trigger.event().0.clone();
    error!("Error Notification: {}", msg);
    
    let id = queue.next_id;
    queue.next_id += 1;
    
    queue.messages.push(Notification {
        message: msg,
        timer: Timer::from_seconds(10.0, TimerMode::Once), // Keep 10s as per original
        id,
    });
}

pub fn notification_lifecycle(time: Res<Time>, mut queue: ResMut<NotificationQueue>) {
    for note in &mut queue.messages {
        note.timer.tick(time.delta());
    }
    queue.messages.retain(|n| !n.timer.is_finished());
}

pub fn ui_notification_system(mut egui: EguiContexts, mut queue: ResMut<NotificationQueue>) {
    let Ok(ctx) = egui.ctx_mut() else {
        return;
    };
    
    // Show a top-right or bottom-right window that is transparent/frameless
    egui::Window::new("Notifications")
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::Vec2::new(-10.0, -10.0))
        .resizable(false)
        .title_bar(false)
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
             // We iterate in reverse to show newest at bottom (or top depending on preference)
             // Let's stack them.
             
             let mut to_remove = Vec::new();

             for note in &mut queue.messages {
                 // Custom frame for each notification
                 egui::Frame::window(ui.style())
                    .fill(egui::Color32::from_rgb(50, 0, 0)) // Reddish background
                    .stroke(egui::Stroke::new(1.0, egui::Color32::RED))
                    .inner_margin(10.0) // Try f32 directly or Margin::same(10.0) if i8 fix needed
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&note.message)
                                .color(egui::Color32::WHITE)
                                .strong()
                            );
                            if ui.button("X").clicked() {
                                to_remove.push(note.id);
                            }
                        });
                    });
                 ui.add_space(5.0);
             }

             if !to_remove.is_empty() {
                 queue.messages.retain(|n| !to_remove.contains(&n.id));
             }
        });
}