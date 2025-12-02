use bevy::prelude::*;

// --- EVENTS (User Requests) ---

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestHostStart;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestHostStop;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestHostGoPublic;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestHostGoPrivate;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestResetToMenu;
