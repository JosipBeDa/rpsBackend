use super::game::RPS;
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, collections::HashSet};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RPSError {
    message: &'static str,
}
impl Display for RPSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "There was an error: {}", self)
    }
}

/// Represents the type of messages the rps manager accepts
#[derive(MessageResponse, Debug, Serialize, Deserialize, Clone)]
pub enum RPSData {
    // Client
    Init(Init),
    Action(Action),
    // Server
    State(RPS),
    Update(Update),
    Rooms(Vec<RPS>),
    None
}
impl actix::Message for RPSData {
    type Result = Self;
}

/// Sent to the client
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Update {
    pub game_id: String,
    pub event: Event,
}

/// Indicates the type of event that occurred as a consequence of a player's action
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Event {
    PlayerConnected(String),
    FastToggled(bool),
    Choices(Vec<(String, char)>),
    Exclude(HashSet<String>),
    Winner(String),
    GG(String)
}

/// Message used to instantiate an rps game
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Init {
    pub host: String,
    pub players: Vec<String>,
    pub gg_score: usize
}

/// Sent by the client
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    pub game_id: String,
    pub sender_id: String,
    pub action: RPSAction,
}

/// Data sent by the client indicating the specified action
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RPSAction {
    /// Inserts the player into the rps connections
    Join,
    /// Maps a player to their choice
    Choose(char),
    /// Toggles fast mode
    FastMode(bool),
}