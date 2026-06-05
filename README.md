# ternary-arena: Multi-agent competition arena for {-1, 0, +1} systems

## Why This Exists

When you want intelligences to compete — a card game, a D&D battle, a debate scoring system — you need a structured arena where agents with ternary strategies face off under defined rules, and you can track who wins over time. This crate provides the competition layer: match execution, tournament brackets, scoreboards, and spectator observation. The "card game layer" Casey described where intelligences compete for wits.

## Core Concepts

**Balanced ternary**: Three values: -1 (Neg), 0 (Zero), +1 (Pos). The domain of agent moves.

**Agent**: A competitor with a name and a strategy (repeating sequence of Trits). At each round, the agent plays the next Trit in its strategy, wrapping around.

**ArenaRules**: Defines how pairs of moves resolve into points. Default is rock-paper-scissors-style: Pos beats Neg, Neg beats Zero, Zero beats Pos. Draws score 1 point each. Fully configurable.

**Match**: Two agents compete over a fixed number of rounds. Each round produces a RoundResult with moves and points. After all rounds, a winner is determined (or it's a draw).

**Tournament**: A single-elimination bracket. Agents are paired, matches are played, winners advance. Odd agents get byes.

**ScoreBoard**: Accumulates points, wins, losses, and draws across multiple matches. Produces a leaderboard sorted by total points.

**ArenaSpectator**: Observes completed matches without participating. Records observations and can compute win rates for specific agents across all observed matches.

**Arena**: The top-level environment that combines rules, scoreboard, and spectator. Run matches and tournaments through it.

## Quick Start

```toml
[dependencies]
ternary-arena = "0.1"
```

```rust
use ternary_arena::*;

let a = Agent::new(1, "aggressive", vec![Trit::Pos, Trit::Pos, Trit::Neg]);
let b = Agent::new(2, "balanced", vec![Trit::Zero, Trit::Pos, Trit::Neg]);

let mut arena = Arena::new("colosseum");
let m = arena.play_match(Match::new(a, b, 3));
println!("Winner: {:?}", m.winner());
println!("Leaderboard: {:?}", arena.scoreboard.leaderboard());
```

## API Overview

| Type | Description |
|------|-------------|
| `Trit` | Balanced ternary value: Neg, Zero, or Pos |
| `Agent` | A competitor with ID, name, and move strategy |
| `ArenaRules` | Resolves pairs of moves into points |
| `Match` | A contest between two agents over N rounds |
| `RoundResult` | One round's moves and points |
| `Tournament` | Single-elimination bracket across multiple agents |
| `ScoreBoard` | Tracks points, wins, losses, draws across matches |
| `ArenaSpectator` | Observes matches and computes analytics |
| `Arena` | Top-level environment combining rules, scoring, observation |

## How It Works

Matches are played synchronously: each round, both agents produce their move (from their strategy at the current round index), and the ArenaRules resolve the pair into points. Results accumulate in a Vec of RoundResults.

Tournaments use a simple bracket: agents are paired sequentially (indices 0+1, 2+3, etc.), matches are played, winners advance. If there's an odd number of agents, the last one gets a bye. This repeats until one champion remains. Draws in a match are resolved by advancing the first-listed agent as a tiebreak.

The ScoreBoard uses HashMaps keyed by agent ID, so it works across multiple matches and tournaments. The Spectator stores MatchObservations (who played, who won, total points) and computes win rates on demand.

## Known Limitations

- Tournament brackets are fixed sequential pairing. No seeding, no Swiss-style, no double elimination.
- Draw tiebreak always favors the first-listed agent (agent_a). Not configurable.
- Agents with empty strategies always play Zero. No way to signal "no move."
- No time limits or round limits on matches — all rounds play to completion.
- ArenaSpectator stores all observations in memory with no limit.
- No team-based or N-player matches. Only 1v1.

## Use Cases

- **Strategy comparison**: Pit different ternary strategies against each other and see which wins under various rule sets.
- **Tournament simulation**: Run elimination brackets with many agents to find dominant strategies.
- **Learning from observation**: Use ArenaSpectator to track which strategies perform best and feed that into strategy optimization.
- **Game balance testing**: Define custom ArenaRules and test whether they produce balanced outcomes.

## Ecosystem Context

Part of the SuperInstance ternary ecosystem. Could use `ternary-dice` to generate stochastic strategies, `ternary-agent` for more sophisticated agent implementations, and `ternary-scoring` for alternative scoring algorithms. The competition layer for `ternary-game-theory` and `ternary-evolution-advanced`.

## License

MIT

## See Also
- **ternary-adversarial** — related
- **ternary-games** — related
- **ternary-game-theory** — related
- **ternary-scoring** — related
- **ternary-fitness** — related

