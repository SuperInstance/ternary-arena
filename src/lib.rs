#![forbid(unsafe_code)]

//! Multi-agent competition arena for balanced ternary systems.
//!
//! Provides an Arena where agents (represented by ternary strategies) compete in
//! Matches, organized into Tournaments. A ScoreBoard tracks results, ArenaRules
//! define the ternary rule system, and ArenaSpectators can observe and learn
//! from completed matches.

use std::collections::HashMap;

/// A single balanced ternary value: -1, 0, or +1.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trit {
    Neg,
    Zero,
    Pos,
}

impl Trit {
    pub fn value(self) -> i8 {
        match self {
            Trit::Neg => -1,
            Trit::Zero => 0,
            Trit::Pos => 1,
        }
    }

    pub fn from_i8(v: i8) -> Option<Self> {
        match v {
            -1 => Some(Trit::Neg),
            0 => Some(Trit::Zero),
            1 => Some(Trit::Pos),
            _ => None,
        }
    }
}

/// An agent with a name and a strategy (a sequence of trit moves).
#[derive(Clone, Debug)]
pub struct Agent {
    pub id: u64,
    pub name: String,
    pub strategy: Vec<Trit>,
}

impl Agent {
    pub fn new(id: u64, name: &str, strategy: Vec<Trit>) -> Self {
        Agent { id, name: name.to_string(), strategy }
    }

    pub fn move_at(&self, round: usize) -> Trit {
        if self.strategy.is_empty() {
            return Trit::Zero;
        }
        self.strategy.get(round % self.strategy.len()).copied().unwrap_or(Trit::Zero)
    }

    pub fn strategy_len(&self) -> usize {
        self.strategy.len()
    }
}

/// The ternary rule system governing how moves resolve.
#[derive(Clone, Debug)]
pub struct ArenaRules {
    /// Points awarded to each combination (agent_move, opponent_move) -> agent_points.
    /// Default: win=3, draw=1, loss=0.
    pub points: HashMap<(Trit, Trit), i32>,
}

impl ArenaRules {
    pub fn new() -> Self {
        let mut points = HashMap::new();
        // Rock-paper-scissors style: Pos beats Neg, Neg beats Zero, Zero beats Pos
        points.insert((Trit::Pos, Trit::Neg), 3);  // Pos beats Neg
        points.insert((Trit::Neg, Trit::Zero), 3);  // Neg beats Zero
        points.insert((Trit::Zero, Trit::Pos), 3);  // Zero beats Pos
        points.insert((Trit::Pos, Trit::Pos), 1);   // Draw
        points.insert((Trit::Neg, Trit::Neg), 1);
        points.insert((Trit::Zero, Trit::Zero), 1);
        points.insert((Trit::Pos, Trit::Zero), 0);  // Pos loses to Zero
        points.insert((Trit::Neg, Trit::Pos), 0);   // Neg loses to Pos
        points.insert((Trit::Zero, Trit::Neg), 0);  // Zero loses to Neg
        ArenaRules { points }
    }

    pub fn resolve(&self, a: Trit, b: Trit) -> (i32, i32) {
        let a_pts = self.points.get(&(a, b)).copied().unwrap_or(1);
        let b_pts = self.points.get(&(b, a)).copied().unwrap_or(1);
        (a_pts, b_pts)
    }

    /// Create rules where same move always draws (flat rules).
    pub fn flat() -> Self {
        let mut points = HashMap::new();
        for a in [Trit::Neg, Trit::Zero, Trit::Pos] {
            for b in [Trit::Neg, Trit::Zero, Trit::Pos] {
                let pts = if a == b { 1 } else { 0 };
                points.insert((a, b), pts);
            }
        }
        ArenaRules { points }
    }
}

impl Default for ArenaRules {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single round within a match.
#[derive(Clone, Debug, PartialEq)]
pub struct RoundResult {
    pub round: usize,
    pub a_move: Trit,
    pub b_move: Trit,
    pub a_points: i32,
    pub b_points: i32,
}

/// A match between two agents over a fixed number of rounds.
#[derive(Clone, Debug)]
pub struct Match {
    pub agent_a: Agent,
    pub agent_b: Agent,
    pub rounds: usize,
    pub results: Vec<RoundResult>,
    pub finished: bool,
}

impl Match {
    pub fn new(agent_a: Agent, agent_b: Agent, rounds: usize) -> Self {
        Match { agent_a, agent_b, rounds, results: Vec::new(), finished: false }
    }

    /// Play all rounds and return (a_total, b_total).
    pub fn play(&mut self, rules: &ArenaRules) -> (i32, i32) {
        self.results.clear();
        let mut a_total = 0;
        let mut b_total = 0;
        for round in 0..self.rounds {
            let a_move = self.agent_a.move_at(round);
            let b_move = self.agent_b.move_at(round);
            let (a_pts, b_pts) = rules.resolve(a_move, b_move);
            a_total += a_pts;
            b_total += b_pts;
            self.results.push(RoundResult {
                round,
                a_move,
                b_move,
                a_points: a_pts,
                b_points: b_pts,
            });
        }
        self.finished = true;
        (a_total, b_total)
    }

    pub fn winner(&self) -> Option<u64> {
        if !self.finished {
            return None;
        }
        let a_total: i32 = self.results.iter().map(|r| r.a_points).sum();
        let b_total: i32 = self.results.iter().map(|r| r.b_points).sum();
        if a_total > b_total {
            Some(self.agent_a.id)
        } else if b_total > a_total {
            Some(self.agent_b.id)
        } else {
            None // draw
        }
    }

    pub fn total_points(&self) -> (i32, i32) {
        let a: i32 = self.results.iter().map(|r| r.a_points).sum();
        let b: i32 = self.results.iter().map(|r| r.b_points).sum();
        (a, b)
    }
}

/// Tracks scores across multiple matches.
#[derive(Clone, Debug)]
pub struct ScoreBoard {
    scores: HashMap<u64, i32>,
    wins: HashMap<u64, u32>,
    losses: HashMap<u64, u32>,
    draws: HashMap<u64, u32>,
}

impl ScoreBoard {
    pub fn new() -> Self {
        ScoreBoard {
            scores: HashMap::new(),
            wins: HashMap::new(),
            losses: HashMap::new(),
            draws: HashMap::new(),
        }
    }

    pub fn record(&mut self, m: &Match) {
        if !m.finished {
            return;
        }
        let (a_total, b_total) = m.total_points();
        *self.scores.entry(m.agent_a.id).or_insert(0) += a_total;
        *self.scores.entry(m.agent_b.id).or_insert(0) += b_total;

        match m.winner() {
            Some(winner_id) => {
                *self.wins.entry(winner_id).or_insert(0) += 1;
                let loser_id = if winner_id == m.agent_a.id { m.agent_b.id } else { m.agent_a.id };
                *self.losses.entry(loser_id).or_insert(0) += 1;
            }
            None => {
                *self.draws.entry(m.agent_a.id).or_insert(0) += 1;
                *self.draws.entry(m.agent_b.id).or_insert(0) += 1;
            }
        }
    }

    pub fn score(&self, id: u64) -> i32 {
        self.scores.get(&id).copied().unwrap_or(0)
    }

    pub fn wins(&self, id: u64) -> u32 {
        self.wins.get(&id).copied().unwrap_or(0)
    }

    pub fn losses(&self, id: u64) -> u32 {
        self.losses.get(&id).copied().unwrap_or(0)
    }

    pub fn draws(&self, id: u64) -> u32 {
        self.draws.get(&id).copied().unwrap_or(0)
    }

    pub fn leaderboard(&self) -> Vec<(u64, i32)> {
        let mut board: Vec<(u64, i32)> = self.scores.iter().map(|(&id, &s)| (id, s)).collect();
        board.sort_by(|a, b| b.1.cmp(&a.1));
        board
    }
}

impl Default for ScoreBoard {
    fn default() -> Self {
        Self::new()
    }
}

/// A multi-round elimination tournament.
#[derive(Clone, Debug)]
pub struct Tournament {
    pub name: String,
    pub agents: Vec<Agent>,
    pub rounds_per_match: usize,
    pub bracket: Vec<Match>,
    pub finished: bool,
}

impl Tournament {
    pub fn new(name: &str, agents: Vec<Agent>, rounds_per_match: usize) -> Self {
        Tournament {
            name: name.to_string(),
            agents,
            rounds_per_match,
            bracket: Vec::new(),
            finished: false,
        }
    }

    /// Run the full tournament: pair agents, play matches, winners advance.
    /// Returns the champion's agent ID (or None for a draw in the final).
    pub fn run(&mut self, rules: &ArenaRules) -> Option<u64> {
        let mut current_agents: Vec<Agent> = self.agents.clone();
        self.bracket.clear();

        while current_agents.len() > 1 {
            let mut winners = Vec::new();
            let mut i = 0;
            while i + 1 < current_agents.len() {
                let b = current_agents[i + 1].clone();
                let a = current_agents[i].clone();
                let mut m = Match::new(a, b, self.rounds_per_match);
                m.play(rules);
                self.bracket.push(m.clone());

                let winner_id = m.winner();
                if let Some(wid) = winner_id {
                    let winner = current_agents[i..i + 2]
                        .iter()
                        .find(|a| a.id == wid)
                        .cloned()
                        .unwrap();
                    winners.push(winner);
                } else {
                    // Draw: first agent advances by tiebreak
                    winners.push(current_agents[i].clone());
                }
                i += 2;
            }
            // Odd agent gets a bye
            if i < current_agents.len() {
                winners.push(current_agents[i].clone());
            }
            current_agents = winners;
        }

        self.finished = true;
        if current_agents.len() == 1 {
            Some(current_agents[0].id)
        } else {
            None
        }
    }

    pub fn match_count(&self) -> usize {
        self.bracket.len()
    }
}

/// An observer that records match outcomes for analysis.
#[derive(Clone, Debug)]
pub struct ArenaSpectator {
    pub name: String,
    observations: Vec<MatchObservation>,
}

#[derive(Clone, Debug)]
pub struct MatchObservation {
    pub agent_a_id: u64,
    pub agent_b_id: u64,
    pub winner_id: Option<u64>,
    pub a_total: i32,
    pub b_total: i32,
}

impl ArenaSpectator {
    pub fn new(name: &str) -> Self {
        ArenaSpectator {
            name: name.to_string(),
            observations: Vec::new(),
        }
    }

    pub fn observe(&mut self, m: &Match) {
        if !m.finished {
            return;
        }
        let (a_total, b_total) = m.total_points();
        self.observations.push(MatchObservation {
            agent_a_id: m.agent_a.id,
            agent_b_id: m.agent_b.id,
            winner_id: m.winner(),
            a_total,
            b_total,
        });
    }

    pub fn observation_count(&self) -> usize {
        self.observations.len()
    }

    pub fn observations(&self) -> &[MatchObservation] {
        &self.observations
    }

    /// Get win rate for a specific agent across all observed matches.
    pub fn win_rate(&self, agent_id: u64) -> f64 {
        let participated: Vec<_> = self.observations
            .iter()
            .filter(|o| o.agent_a_id == agent_id || o.agent_b_id == agent_id)
            .collect();
        if participated.is_empty() {
            return 0.0;
        }
        let wins = participated.iter().filter(|o| o.winner_id == Some(agent_id)).count();
        wins as f64 / participated.len() as f64
    }
}

/// The competition environment that orchestrates matches and tracks standings.
#[derive(Clone, Debug)]
pub struct Arena {
    pub name: String,
    pub rules: ArenaRules,
    pub scoreboard: ScoreBoard,
    pub spectator: ArenaSpectator,
}

impl Arena {
    pub fn new(name: &str) -> Self {
        Arena {
            name: name.to_string(),
            rules: ArenaRules::new(),
            scoreboard: ScoreBoard::new(),
            spectator: ArenaSpectator::new("arena-spectator"),
        }
    }

    pub fn with_rules(name: &str, rules: ArenaRules) -> Self {
        Arena {
            name: name.to_string(),
            rules,
            scoreboard: ScoreBoard::new(),
            spectator: ArenaSpectator::new("arena-spectator"),
        }
    }

    /// Run a match in-place, recording the result.
    pub fn run_match_mut(&mut self, m: &mut Match) {
        m.play(&self.rules);
        self.scoreboard.record(m);
        self.spectator.observe(m);
    }

    /// Run a match and return it. Records to scoreboard and spectator.
    pub fn play_match(&mut self, mut m: Match) -> Match {
        m.play(&self.rules);
        self.scoreboard.record(&m);
        self.spectator.observe(&m);
        m
    }

    /// Run a tournament in this arena.
    pub fn play_tournament(&mut self, mut tournament: Tournament) -> Option<u64> {
        let champion = tournament.run(&self.rules);
        for m in &tournament.bracket {
            self.scoreboard.record(m);
            self.spectator.observe(m);
        }
        champion
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_move_at() {
        let agent = Agent::new(1, "test", vec![Trit::Pos, Trit::Neg]);
        assert_eq!(agent.move_at(0), Trit::Pos);
        assert_eq!(agent.move_at(1), Trit::Neg);
        assert_eq!(agent.move_at(2), Trit::Pos); // wraps
    }

    #[test]
    fn agent_move_at_empty_strategy() {
        let agent = Agent::new(1, "test", vec![]);
        assert_eq!(agent.move_at(0), Trit::Zero);
    }

    #[test]
    fn arena_rules_default_resolve() {
        let rules = ArenaRules::new();
        // Pos beats Neg
        assert_eq!(rules.resolve(Trit::Pos, Trit::Neg), (3, 0));
        // Draw
        assert_eq!(rules.resolve(Trit::Pos, Trit::Pos), (1, 1));
        // Zero beats Pos
        assert_eq!(rules.resolve(Trit::Zero, Trit::Pos), (3, 0));
    }

    #[test]
    fn arena_rules_flat() {
        let rules = ArenaRules::flat();
        // All same-move = draw (1,1)
        assert_eq!(rules.resolve(Trit::Pos, Trit::Pos), (1, 1));
        // Different moves = (0, 0)
        assert_eq!(rules.resolve(Trit::Pos, Trit::Neg), (0, 0));
    }

    #[test]
    fn match_play_three_rounds() {
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Neg]);
        let mut m = Match::new(a, b, 3);
        let (a_total, b_total) = m.play(&ArenaRules::new());
        // Pos beats Neg every round: 3*3 = 9 for a, 0 for b
        assert_eq!(a_total, 9);
        assert_eq!(b_total, 0);
        assert_eq!(m.results.len(), 3);
        assert!(m.finished);
    }

    #[test]
    fn match_winner() {
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Neg]);
        let mut m = Match::new(a, b, 1);
        m.play(&ArenaRules::new());
        assert_eq!(m.winner(), Some(1));
    }

    #[test]
    fn match_draw() {
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Pos]);
        let mut m = Match::new(a, b, 1);
        m.play(&ArenaRules::new());
        assert_eq!(m.winner(), None);
    }

    #[test]
    fn match_winner_before_play() {
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Neg]);
        let m = Match::new(a, b, 1);
        assert_eq!(m.winner(), None); // not finished
    }

    #[test]
    fn scoreboard_records_match() {
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Neg]);
        let mut m = Match::new(a, b, 3);
        m.play(&ArenaRules::new());

        let mut sb = ScoreBoard::new();
        sb.record(&m);
        assert_eq!(sb.score(1), 9);
        assert_eq!(sb.score(2), 0);
        assert_eq!(sb.wins(1), 1);
        assert_eq!(sb.losses(2), 1);
    }

    #[test]
    fn scoreboard_leaderboard() {
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Neg]);
        let mut m = Match::new(a, b, 1);
        m.play(&ArenaRules::new());

        let mut sb = ScoreBoard::new();
        sb.record(&m);
        let board = sb.leaderboard();
        assert_eq!(board[0].0, 1); // a has more points
    }

    #[test]
    fn scoreboard_unregistered_agent() {
        let sb = ScoreBoard::new();
        assert_eq!(sb.score(999), 0);
        assert_eq!(sb.wins(999), 0);
    }

    #[test]
    fn tournament_run_power_of_two() {
        let agents = vec![
            Agent::new(1, "a", vec![Trit::Pos]),
            Agent::new(2, "b", vec![Trit::Neg]),
            Agent::new(3, "c", vec![Trit::Zero]),
            Agent::new(4, "d", vec![Trit::Pos]),
        ];
        let mut t = Tournament::new("test", agents, 1);
        let champion = t.run(&ArenaRules::new());
        assert!(champion.is_some());
        assert!(t.finished);
        assert_eq!(t.match_count(), 3); // 2 semifinals + 1 final
    }

    #[test]
    fn tournament_odd_agents() {
        let agents = vec![
            Agent::new(1, "a", vec![Trit::Pos]),
            Agent::new(2, "b", vec![Trit::Neg]),
            Agent::new(3, "c", vec![Trit::Zero]),
        ];
        let mut t = Tournament::new("test", agents, 1);
        let champion = t.run(&ArenaRules::new());
        assert!(champion.is_some());
        assert_eq!(t.match_count(), 2); // 1 match + bye, then 1 final
    }

    #[test]
    fn spectator_observe() {
        let mut spec = ArenaSpectator::new("observer");
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Neg]);
        let mut m = Match::new(a, b, 1);
        m.play(&ArenaRules::new());
        spec.observe(&m);
        assert_eq!(spec.observation_count(), 1);
        assert_eq!(spec.observations()[0].winner_id, Some(1));
    }

    #[test]
    fn spectator_win_rate() {
        let mut spec = ArenaSpectator::new("observer");
        // Agent 1 wins
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Neg]);
        let mut m1 = Match::new(a, b, 1);
        m1.play(&ArenaRules::new());
        spec.observe(&m1);
        // Agent 1 loses
        let c = Agent::new(1, "a", vec![Trit::Neg]);
        let d = Agent::new(3, "c", vec![Trit::Pos]);
        let mut m2 = Match::new(c, d, 1);
        m2.play(&ArenaRules::new());
        spec.observe(&m2);

        assert!((spec.win_rate(1) - 0.5).abs() < 0.001);
    }

    #[test]
    fn spectator_no_observations() {
        let spec = ArenaSpectator::new("observer");
        assert_eq!(spec.win_rate(1), 0.0);
    }

    #[test]
    fn arena_play_match() {
        let mut arena = Arena::new("colosseum");
        let a = Agent::new(1, "a", vec![Trit::Pos]);
        let b = Agent::new(2, "b", vec![Trit::Neg]);
        let m = Match::new(a, b, 3);
        let result = arena.play_match(m);
        assert!(result.finished);
        assert_eq!(arena.scoreboard.score(1), 9);
        assert_eq!(arena.spectator.observation_count(), 1);
    }

    #[test]
    fn arena_play_tournament() {
        let mut arena = Arena::new("colosseum");
        let agents = vec![
            Agent::new(1, "a", vec![Trit::Pos]),
            Agent::new(2, "b", vec![Trit::Neg]),
        ];
        let t = Tournament::new("cup", agents, 1);
        let champion = arena.play_tournament(t);
        assert_eq!(champion, Some(1));
    }

    #[test]
    fn round_result_fields() {
        let rr = RoundResult {
            round: 5,
            a_move: Trit::Pos,
            b_move: Trit::Neg,
            a_points: 3,
            b_points: 0,
        };
        assert_eq!(rr.round, 5);
        assert_eq!(rr.a_points, 3);
    }

    #[test]
    fn agent_strategy_len() {
        let agent = Agent::new(1, "a", vec![Trit::Pos, Trit::Neg, Trit::Zero]);
        assert_eq!(agent.strategy_len(), 3);
    }
}
