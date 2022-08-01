use super::models::{Event, RPSData, Update};
use crate::actors::{
    db::{manager::DBManager, messages::StoreHoFEntry},
    ez_handler,
    models::messages::{
        client_message::{MessageData, SocketMessage},
        connection::Connect,
    },
};
use actix::prelude::*;
use actix::Actor;
use colored::Colorize;
use std::collections::HashMap;
use tracing::info;
use tracing::log::warn;
use uuid::Uuid;

use super::game::{RpsResolve, RPS};
use super::models::RPSAction;

/// An actor that maintains the state of all RPS games
pub struct RPSManager {
    sessions: HashMap<String, Recipient<SocketMessage>>,
    games: HashMap<String, RPS>,
    db_manager: Addr<DBManager>,
}

impl RPSManager {
    pub fn new(db_manager: Addr<DBManager>) -> Self {
        Self {
            sessions: HashMap::new(),
            games: HashMap::new(),
            db_manager,
        }
    }

    /// Register a new RPS game with the given players and the given player id as the host
    pub fn register_game(&mut self, players: Vec<String>, host: String, gg_score: usize) -> RPS {
        let id = Uuid::new_v4().to_string();
        self.games
            .insert(id.clone(), RPS::new(players, host, &id, gg_score));
        info!("{}{:?}", "ACTIVE GAMES : ".purple(), self.games);
        let game = self.games.get(&id).unwrap().clone();
        self.broadcast(&game);
        game
    }

    /// Returns all registered games
    fn get_games(&self) -> Vec<RPS> {
        self.games.values().cloned().collect()
    }

    /// Directly send a message to the address of the given session ID.
    fn send_direct(&self, receiver: &str, message: String) {
        if let Some(address) = self.sessions.get(receiver) {
            let _ = address.do_send(SocketMessage(message));
        }
    }

    /// Broadcasts a message to all sessions connected to the RPS manager.
    pub fn broadcast(&self, rps: &RPS) {
        info!("BROADCASTING TO : {:?}", self.sessions.keys());
        for (_, address) in &self.sessions {
            address.do_send(SocketMessage(
                ez_handler::generate_message::<RPS>(
                    "rps",
                    MessageData::RPS(RPSData::State(rps.clone())),
                )
                .unwrap(),
            ));
        }
    }

    pub fn room_broadcast(&self, game: &RPS, data: RPSData) {
        for (id, address) in &self.sessions {
            if game.connections.contains(id) {
                address.do_send(SocketMessage(
                    ez_handler::generate_message::<RPS>("rps", MessageData::RPS(data.clone()))
                        .unwrap(),
                ))
            }
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
            RPSData::Init(msg) => {
                RPSData::State(self.register_game(msg.players, msg.host, msg.gg_score))
            }
            RPSData::Action(msg) => {
                let game = self.games.get_mut(&msg.game_id).unwrap();
                match msg.action {
                    RPSAction::Join => {
                        // Only send the state when the player is joining the game for the first time
                        if game.player_ids.contains(&msg.sender_id)
                            && !game.connections.contains(&msg.sender_id)
                        {
                            game.connections.insert(msg.sender_id.clone());

                            let game = self.games.get(&msg.game_id).unwrap();
                            self.room_broadcast(
                                &game,
                                RPSData::Update(Update {
                                    game_id: game.id.clone(),
                                    event: Event::PlayerConnected(msg.sender_id.clone()),
                                }),
                            );
                            return RPSData::State(game.clone());
                        }
                        RPSData::None
                    }
                    RPSAction::FastMode(flag) => {
                        if msg.sender_id == game.host {
                            game.toggle_fast(flag);
                        }

                        let game = self.games.get(&msg.game_id).unwrap();
                        self.room_broadcast(
                            &game,
                            RPSData::Update(Update {
                                game_id: game.id.clone(),
                                event: Event::FastToggled(game.fast_mode),
                            }),
                        );
                        RPSData::None
                    }
                    RPSAction::Choose(rps) => {
                        // If the game can be resolved
                        if let Some(resolve) = game.choose_rps(rps, msg.sender_id.clone()) {
                            // Drain choices from game
                            let choices: Vec<(String, char)> = game.choices.drain().collect();
                            match resolve {
                                RpsResolve::Exclude(losers) => {
                                    // Shadow the mutable reference
                                    let game = self.games.get(&msg.game_id).unwrap();
                                    // Broadcast choices
                                    self.room_broadcast(
                                        &game,
                                        RPSData::Update(Update {
                                            game_id: game.id.clone(),
                                            event: Event::Choices(choices.clone()),
                                        }),
                                    );
                                    // Broadcast losers
                                    self.room_broadcast(
                                        &game,
                                        RPSData::Update(Update {
                                            game_id: game.id.clone(),
                                            event: Event::Exclude(losers.clone()),
                                        }),
                                    );
                                    return RPSData::None;
                                }
                                RpsResolve::Winner(winner) => {
                                    game.reset_excluded();
                                    // Shadow the mutable reference
                                    let game = self.games.get(&msg.game_id).unwrap();
                                    // Broadcast choices
                                    self.room_broadcast(
                                        &game,
                                        RPSData::Update(Update {
                                            game_id: game.id.clone(),
                                            event: Event::Choices(choices.clone()),
                                        }),
                                    );
                                    // Broadcast winner
                                    self.room_broadcast(
                                        &game,
                                        RPSData::Update(Update {
                                            game_id: game.id.clone(),
                                            event: Event::Winner(winner.clone()),
                                        }),
                                    );

                                    // If the score threshold is reached
                                    if game.scores.get(&winner).unwrap() >= &game.gg_score {
                                        info!("MAXIMUM SCORE REACHED -- WINNER : {:?}", winner);
                                        let game = self.games.get_mut(&msg.game_id).unwrap();
                                        game.end();

                                        self.db_manager.do_send(StoreHoFEntry { user_id: winner });

                                        let game = self.games.get(&msg.game_id).unwrap();
                                        self.room_broadcast(
                                            &game,
                                            RPSData::Update(Update {
                                                game_id: game.id.clone(),
                                                event: Event::GG(game.id.clone()),
                                            }),
                                        );
                                    }
                                    return RPSData::None;
                                }
                            }
                        }
                        RPSData::None
                    }
                }
            }
            _ => {
                warn!("RPS BAD MESSAGE!");
                RPSData::None
            }
        }
    }
}
