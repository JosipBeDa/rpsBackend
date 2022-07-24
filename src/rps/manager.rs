use crate::chat::ez_handler;
use crate::chat::models::messages::{Connect, Message, MessageData};
use crate::rps::models::Event;
use crate::rps::models::RPSData;
use crate::rps::models::Update;
use actix::prelude::*;
use actix::Actor;
use colored::Colorize;
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::info;
use tracing::log::warn;
use uuid::Uuid;

use super::game::RPS;
use super::models::RPSAction;

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

    /// Register a new RPS game with the given players and the given player id as the host
    pub fn register_game(&mut self, players: Vec<String>, host: String) -> RPS {
        let id = Uuid::new_v4().to_string();
        self.games
            .insert(id.clone(), RPS::new(players, host, id.clone()));
        info!("{}{:?}", "ACTIVE GAMES : ".purple(), self.games);

        let game = self.games.get(&id).unwrap().clone();
        self.broadcast(game.clone());
        game
    }

    /// Returns all registered games
    fn get_games(&self) -> Vec<RPS> {
        self.games.values().cloned().collect()
    }

    /// Directly send a message to the address of the given session ID.
    fn send_direct(&self, receiver: &str, message: String) {
        if let Some(address) = self.sessions.get(receiver) {
            let _ = address.do_send(Message(message.clone()));
        }
    }

    /// Broadcasts a message to all sessions connected to the RPS manager.
    pub fn broadcast(&self, rps: RPS) {
        info!("BROADCASTING TO : {:?}", self.sessions.keys());
        for (_, address) in &self.sessions {
            address.do_send(Message(
                ez_handler::generate_message::<RPS>(
                    "rps",
                    MessageData::RPS(RPSData::State(rps.clone())),
                )
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
    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        self.sessions.insert(msg.user.id.clone(), msg.address);
        // Send all active games to the user
        self.send_direct(
            &msg.user.id,
            ez_handler::generate_message::<RPS>(
                "rps",
                MessageData::RPS(RPSData::Rooms(self.get_games())),
            )
            .unwrap(),
        )
    }
}

impl Handler<RPSData> for RPSManager {
    type Result = RPSData;
    fn handle(&mut self, msg: RPSData, _: &mut Self::Context) -> Self::Result {
        match msg {
            RPSData::Init(msg) => RPSData::State(self.register_game(msg.players, msg.host)),
            RPSData::Action(msg) => {
                let game = self.games.get_mut(&msg.game_id).unwrap();
                match msg.action {
                    RPSAction::Join => {
                        if game.player_ids.contains(&msg.sender_id)
                            && !game.connections.contains(&msg.sender_id)
                        {
                            // Send update to all connected players
                            game.connections.insert(msg.sender_id.clone());
                            for (id, address) in &self.sessions {
                                if game.connections.contains(id) && !id.eq(&msg.sender_id) {
                                    address.do_send(Message(
                                        ez_handler::generate_message::<RPS>(
                                            "rps",
                                            MessageData::RPS(RPSData::Update(Update {
                                                game_id: game.id.clone(),
                                                event: Event::PlayerConnected(
                                                    msg.sender_id.clone(),
                                                ),
                                            })),
                                        )
                                        .unwrap(),
                                    ));
                                }
                            }
                        }
                        RPSData::State(game.clone())
                    }
                    RPSAction::FastMode(flag) => {
                        if msg.sender_id == game.host {
                            game.toggle_fast(flag);
                        }
                        for (id, address) in &self.sessions {
                            if game.connections.contains(id) {
                                address.do_send(Message(
                                    ez_handler::generate_message::<RPS>(
                                        "rps",
                                        MessageData::RPS(RPSData::Update(Update {
                                            game_id: game.id.clone(),
                                            event: Event::FastToggled(game.fast_mode),
                                        })),
                                    )
                                    .unwrap(),
                                ));
                            }
                        }
                        RPSData::None
                    }
                    RPSAction::Spectate => {
                        self.spectators
                            .entry(msg.game_id)
                            .or_insert_with(|| HashSet::new())
                            .insert(msg.sender_id);
                        RPSData::State(game.clone())
                    }
                    RPSAction::Choose(rps) => {
                        // If the game can be resolved
                        if let Some(winners) = game.choose_rps(rps, msg.sender_id.clone()) {
                            for (id, address) in &self.sessions {
                                if game.connections.contains(id) {
                                    // Send the choices so client can show them to everyone
                                    address.do_send(Message(
                                        ez_handler::generate_message::<RPS>(
                                            "rps",
                                            MessageData::RPS(RPSData::Update(Update {
                                                game_id: game.id.clone(),
                                                event: Event::Choices(
                                                    game.choices.clone().drain().collect(),
                                                ),
                                            })),
                                        )
                                        .unwrap(),
                                    ));
                                    // Send winners
                                    address.do_send(Message(
                                        ez_handler::generate_message::<RPS>(
                                            "rps",
                                            MessageData::RPS(RPSData::Update(Update {
                                                game_id: game.id.clone(),
                                                event: Event::Winners(winners.clone()),
                                            })),
                                        )
                                        .unwrap(),
                                    ));
                                }
                            }
                            game.reset_choices();
                            return RPSData::None;
                        }
                        RPSData::None
                    }

                    RPSAction::Reset => todo!(),
                }
            }
            _ => {
                warn!("RPS BAD MESSAGE!");
                RPSData::None
            }
        }
    }
}
