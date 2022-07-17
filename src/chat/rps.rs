use crate::chat::models::Message;
use actix::prelude::*;
use actix::Actor;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Display;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::error::GlobalError;

use super::ez_handler;
use super::models::Connect;
use super::models::MessageData;

/// Keeps track of the state of individual RPS games.
#[derive(MessageResponse, Debug, Serialize, Deserialize, Clone)]
pub struct RPS {
    pub id: String,
    pub host: String,
    pub player_ids: HashSet<String>,
    pub choices: HashMap<String, RPSChoice>,
    pub scores: HashMap<String, u32>,
    pub connections: HashSet<String>,
    pub fast_mode: bool,
    pub locked: bool,
    pub game_over: bool,
}

impl RPS {
    pub fn new(players: Vec<String>, host: String, game_id: String) -> Self {
        let mut player_ids = HashSet::new();
        let mut scores = HashMap::new();
        let mut connections = HashSet::new();
        let choices = HashMap::new();
        for id in players {
            player_ids.insert(id.clone());
            scores.insert(id, 0);
        }
        connections.insert(host.clone());
        Self {
            id: game_id,
            host,
            player_ids,
            scores,
            connections,
            choices,
            fast_mode: false,
            locked: false,
            game_over: false,
        }
    }

    pub fn toggle_fast(&mut self) {
        self.fast_mode = !self.fast_mode;
    }

    pub fn choose_rps(&mut self, rps: char, player_id: String) -> Option<Vec<String>> {
        let choice = self
            .choices
            .entry(player_id)
            .or_insert(RPSChoice::from(rps).unwrap());
        if let Ok(c) = RPSChoice::from(rps) {
            *choice = c;
        }
        if self.choices.values().len() == self.connections.len() {
            let winners = self.resolve_rps();
            Some(winners)
        } else {
            None
        }
    }

    fn resolve_rps(&mut self) -> Vec<String> {
        let mut winners: Vec<String> = vec![];
        let mut results: HashMap<String, i8> = HashMap::new();
        for conn_id in self.connections.clone() {
            let mut score: i8 = 0;
            let player_choice = self.choices.get(&conn_id).unwrap();
            for (_id, choice) in self.choices.clone() {
                match player_choice {
                    RPSChoice::Rock => {
                        if choice == RPSChoice::Scissors {
                            score += 1;
                            continue;
                        }
                        if choice == RPSChoice::Paper {
                            score -= 1;
                        }
                    }
                    RPSChoice::Paper => {
                        if choice == RPSChoice::Rock {
                            score += 1;
                            continue;
                        }
                        if choice == RPSChoice::Scissors {
                            score -= 1;
                        }
                    }
                    RPSChoice::Scissors => {
                        if choice == RPSChoice::Paper {
                            score += 1;
                            continue;
                        }
                        if choice == RPSChoice::Rock {
                            score -= 1;
                        }
                    }
                    RPSChoice::KamenNajjaci => {
                        if choice == RPSChoice::KamenNajjaci {
                            continue;
                        }
                        score += 1;
                    }
                }
            }
            results.insert(conn_id, score);
        }
        info!("RESULTS : {:?}", results);
        let mut max = 0;
        for (id, score) in results {
            if score < max {
                continue;
            }
            if score == max {
                winners.push(id);
                continue;
            }
            if score > max {
                max = score;
                winners.clear();
                winners.push(id);
            }
        }
        info!("WINNERS : {:?}", winners);
        winners
    }

    pub fn reset_choices(&mut self) {
        self.choices.clear();
    }

    pub fn disconnect_player(&mut self, player_id: &str) {
        self.choices.remove(player_id);
        self.connections.remove(player_id);
    }

    fn _end(&mut self) {
        self.game_over = true;
    }
}
/// An actor that maintains the state of all RPS games
pub struct RPSManager {
    sessions: HashMap<String, Recipient<Message>>,
    games: HashMap<String, RPS>,
    spectators: HashMap<String, HashSet<String>>,
}

impl RPSManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            games: HashMap::new(),
            spectators: HashMap::new(),
        }
    }

    pub fn register_game(&mut self, players: Vec<String>, host: String) -> RPS {
        let id = Uuid::new_v4().to_string();
        self.games
            .insert(id.clone(), RPS::new(players, host, id.clone()));
        info!("{}{:?}", "ACTIVE GAMES : ".purple(), self.games);

        let game = self.games.get(&id).unwrap().clone();
        self.broadcast(game.clone());
        game
    }

    pub fn broadcast(&mut self, rps: RPS) {
        info!("BROADCASTING TO : {:?}", self.sessions.keys());
        for address in self.sessions.values().clone() {
            address.do_send(Message(
                ez_handler::generate_message::<RPS>("rps", MessageData::RPSState(rps.clone()))
                    .unwrap(),
            ));
        }
    }
}

impl Actor for RPSManager {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Context<Self>) {
        info!("{}", "Started RPS Manager".green());
    }
}

impl Handler<Connect> for RPSManager {
    type Result = ();
    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user.id, msg.address);
        info!("INSERTED SESSION -- {:?}", self.sessions);
    }
}

impl Handler<RPSMessage> for RPSManager {
    type Result = RPS;
    fn handle(&mut self, msg: RPSMessage, _: &mut Self::Context) -> Self::Result {
        match msg.message {
            RPSData::Init(msg) => self.register_game(msg.players, msg.host),
            RPSData::Action(msg) => {
                let game = self.games.get_mut(&msg.game_id).unwrap();
                match msg.action {
                    RPSAction::Join => {
                        if game.player_ids.contains(&msg.sender_id) {
                            info!(
                                "{}{}{}{}",
                                "Player: ".purple(),
                                msg.sender_id,
                                " joined ".purple(),
                                msg.game_id
                            );
                            game.connections.insert(msg.sender_id);
                        }
                    }
                    RPSAction::FastMode(flag) => {
                        if msg.sender_id == game.host {
                            game.fast_mode = flag;
                        }
                    }
                    RPSAction::Spectate => {
                        self.spectators
                            .entry(msg.game_id)
                            .or_insert_with(|| HashSet::new())
                            .insert(msg.sender_id);
                    }
                    RPSAction::Choose(rps) => {
                        let choice = game
                            .choices
                            .entry(msg.sender_id)
                            .or_insert_with(|| rps.clone());
                        *choice = rps;
                    }
                    RPSAction::Reset => todo!(),
                }
                game.clone()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RPSError {
    message: &'static str,
}
impl Display for RPSError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "There was an error: {}", self)
    }
}

#[derive(Message, Debug, Serialize, Deserialize, Clone)]
#[rtype(result = "RPS")]
pub struct RPSMessage {
    message: RPSData,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RPSData {
    Init(Init),
    Action(Action),
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Init {
    host: String,
    players: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Action {
    game_id: String,
    sender_id: String,
    action: RPSAction,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RPSAction {
    Join,
    Choose(RPSChoice),
    FastMode(bool),
    Spectate,
    Reset,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[repr(u8)]
pub enum RPSChoice {
    Rock = b'r',
    Paper = b'p',
    Scissors = b's',
    KamenNajjaci = b'x',
}

impl Into<char> for RPSChoice {
    fn into(self) -> char {
        match self {
            RPSChoice::Rock => 'r',
            RPSChoice::Paper => 'p',
            RPSChoice::Scissors => 's',
            RPSChoice::KamenNajjaci => 'x',
        }
    }
}
impl RPSChoice {
    fn from(c: char) -> Result<Self, GlobalError> {
        match c {
            'r' => Ok(RPSChoice::Rock),
            'p' => Ok(RPSChoice::Paper),
            's' => Ok(RPSChoice::Scissors),
            'x' => Ok(RPSChoice::KamenNajjaci),
            _ => {
                warn!("Invalid choice!");
                Err(GlobalError::RPSError(RPSError {
                    message: "Invalid choice",
                }))
            }
        }
    }
}
