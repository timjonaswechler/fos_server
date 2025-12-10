use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct ErrorMessage {
    pub text: String,
    pub timeout: Option<Timer>,
}

#[derive(Event)]
pub struct NotifyError(pub String);

impl NotifyError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }
}

pub fn on_notify_error(trigger: On<NotifyError>, mut error_message: ResMut<ErrorMessage>) {
    error_message.text = trigger.event().0.clone();
    // Auto-close after 10 seconds
    error_message.timeout = Some(Timer::from_seconds(10.0, TimerMode::Once));
    error!("Error Message triggered: {}", error_message.text);
}

pub fn error_lifecycle(time: Res<Time>, mut error_message: ResMut<ErrorMessage>) {
    if let Some(timer) = &mut error_message.timeout {
        timer.tick(time.delta());
        if timer.is_finished() {
            error_message.text.clear();
            error_message.timeout = None;
        }
    }
}

pub fn clear_error(mut error_message: ResMut<ErrorMessage>) {
    error_message.text.clear();
    error_message.timeout = None;
}
