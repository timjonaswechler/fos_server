use bevy::prelude::*;

// --- EVENTS (Requests) ---

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestHostStart;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestHostStop;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestHostGoPublic;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestHostGoPrivate;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestClientConnect;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestClientDisconnect;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestClientRetry;

#[derive(Event, Debug, Clone, Copy)]
pub struct RequestResetToMenu;