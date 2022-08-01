use actix::prelude::*;
use colored::Colorize;
use rand;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use tracing::{info, warn};

const EPIC_WORDS: &[&str] = &[
    "Deadly Dispute",
    "Supreme Battle",
    "Ultimate Showdown",
    "Quest for Glory",
];

/// RPS game instance.
#[derive(MessageResponse, Debug, Serialize, Deserialize, Clone)]
pub struct RPS {
    pub id: String,
    pub name: String,
    pub host: String,
    pub player_ids: HashSet<String>,
    pub choices: HashMap<String, char>,
    pub scores: HashMap<String, usize>,
    pub connections: HashSet<String>,
    pub fast_mode: bool,
    pub locked: bool,
    pub excluded: HashSet<String>,
    pub game_over: bool,
    pub gg_score: usize,
}

impl RPS {
    pub fn new(players: Vec<String>, host: String, id: &str, gg_score: usize) -> Self {
        let name = generate_epic_word();
        let choices = HashMap::new();
        let excluded = HashSet::new();
        let mut player_ids = HashSet::new();
        let mut scores = HashMap::new();
        let mut connections = HashSet::new();
        connections.insert(host.clone());
        for id in players {
            player_ids.insert(id.clone());
            scores.insert(id, 0);
        }
        Self {
            id: id.to_string(),
            name,
            host,
            player_ids,
            scores,
            connections,
            choices,
            fast_mode: false,
            locked: false,
            excluded,
            game_over: false,
            gg_score,
        }
    }

    pub fn toggle_fast(&mut self, flag: bool) {
        self.fast_mode = flag;
    }

    pub fn choose_rps(&mut self, rps: char, player_id: String) -> Option<RpsResolve> {
        if self.excluded.contains(&player_id) {
            return None;
        }
        let choice = self.choices.entry(player_id).or_insert(rps);
        *choice = rps;
        if self.choices.values().len() == self.connections.len() - self.excluded.len() {
            let winners = self.resolve_rps();
            Some(winners)
        } else {
            None
        }
    }

    fn resolve_rps(&mut self) -> RpsResolve {
        info!("{}", "------ RESOLVING RPS ------- ".yellow());
        let mut winners: Vec<String> = vec![];
        let mut results: HashMap<String, i8> = HashMap::new();
        for conn_id in self.connections.clone() {
            if !self.excluded.contains(&conn_id) {
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
        // If everyone wins, no one wins
        if winners.len() == self.connections.len() {
            return RpsResolve::Exclude(HashSet::new());
        }
        // If there's more than one winner, exclude the losers and start next round
        if winners.len() > 1 {
            let mut excluded = HashSet::new();
            for id in self.connections.clone() {
                if !winners.contains(&id) && !self.excluded.contains(&id) {
                    self.excluded.insert(id.clone());
                    excluded.insert(id);
                }
            }
            return RpsResolve::Exclude(excluded);
        }
        // If there is exactly one winner increment their score and return their ID
        let winner = winners.pop().unwrap();
        *self.scores.get_mut(&winner[..]).unwrap() += 1;
        info!("WINNER : {:?}", winner);
        RpsResolve::Winner(winner)
    }

    pub fn reset_choices(&mut self) {
        self.choices.clear();
    }

    pub fn reset_excluded(&mut self) {
        self.excluded.clear();
    }

    fn _disconnect_player(&mut self, player_id: &str) {
        self.choices.remove(player_id);
        self.connections.remove(player_id);
    }

    pub fn end(&mut self) {
        self.game_over = true;
    }
}

fn generate_epic_word() -> String {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..EPIC_WORDS.len());
    EPIC_WORDS[idx].to_string()
}

/// Internal type returned by the `resolve_rps()` function
pub enum RpsResolve {
    Exclude(HashSet<String>),
    Winner(String)
}