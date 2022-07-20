use actix::prelude::*;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::{info, warn};

/// RPS game instance.
#[derive(MessageResponse, Debug, Serialize, Deserialize, Clone)]
pub struct RPS {
    pub id: String,
    pub host: String,
    pub player_ids: HashSet<String>,
    pub choices: HashMap<String, char>,
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
        let choice = self.choices.entry(player_id).or_insert(rps);
        *choice = rps;
        if self.choices.values().len() == self.connections.len() {
            let winners = self.resolve_rps();
            Some(winners)
        } else {
            None
        }
    }

    fn resolve_rps(&mut self) -> Vec<String> {
        info!("{}", "------ RESOLVING RPS ------- ".yellow());
        let mut winners: Vec<String> = vec![];
        let mut results: HashMap<String, i8> = HashMap::new();
        for conn_id in self.connections.clone() {
            let mut score: i8 = 0;
            let player_choice = self.choices.get(&conn_id).unwrap();
            for (_id, choice) in self.choices.clone() {
                match player_choice {
                    'r' => {
                        if choice == 's' {
                            score += 1;
                            continue;
                        }
                        if choice == 'p' {
                            score -= 1;
                        }
                    }
                    'p' => {
                        if choice == 'r' {
                            score += 1;
                            continue;
                        }
                        if choice == 's' {
                            score -= 1;
                        }
                    }
                    's' => {
                        if choice == 'p' {
                            score += 1;
                            continue;
                        }
                        if choice == 'r' {
                            score -= 1;
                        }
                    }
                    'x' => {
                        if choice == 'x' {
                            continue;
                        }
                        score += 1;
                    }
                    _ => {
                        warn!("{}", "Got invalid RPS key".red())
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
        // Nobody wins if it's a small game and it's a draw
        if winners.len() > 1 && self.player_ids.len() < 3 {
            return vec![];
        }
        for winner in winners.clone() {
            *self.scores.get_mut(&winner).unwrap() += 1;
        }
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
