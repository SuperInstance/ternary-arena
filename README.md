# Ternary Arena

**Ternary Arena** is a multi-agent competition framework for balanced ternary systems — providing matches, tournaments, scoreboards, rule systems, and spectator abstractions for pitting ternary strategies against each other.

## Why It Matters

Game-theoretic evaluation requires controlled environments where strategies compete under defined rules. Ternary Arena provides this with a ternary twist: instead of binary (win/lose), outcomes use {-1 (loss), 0 (draw), +1 (win)}, enabling richer evaluation including the "abstain" option. The rock-paper-scissors dynamic (Pos beats Neg, Neg beats Zero, Zero beats Pos) creates non-transitive dominance cycles that prevent any single strategy from dominating — promoting diversity.

## How It Works

### Match Mechanics

Two agents face off over R rounds. Each round, both play a `Trit` move from their strategy:

```
agent.move_at(round) → strategy[round % strategy_len]
```

Move resolution via `ArenaRules` payoff matrix (default: rock-paper-scissors):

```
(Pos, Neg) → +3 for Pos  (Pos beats Neg)
(Neg, Zero) → +3 for Neg  (Neg beats Zero)
(Zero, Pos) → +3 for Zero (Zero beats Pos)
(X, X) → +1 each          (draw)
otherwise → 0             (loss)
```

Scoring per round: **O(1)** (HashMap lookup). Match of R rounds: **O(R)**.

### Tournament Structure

A tournament runs all-vs-all round-robin:

```
for i in 0..N:
    for j in 0..N:
        if i != j:
            play_match(agents[i], agents[j])
```

Total matches: N·(N-1). Cost: **O(N² · R)** for N agents, R rounds per match.

### ScoreBoard

Tracks cumulative scores:

```
ScoreBoard {
    scores: HashMap<agent_id, i32>,
    wins, losses, draws per agent
}
```

Update per match: **O(1)**. Ranking: **O(N log N)** (sort by score).

### Spectator Pattern

`ArenaSpectator` trait for observers:

```rust
trait ArenaSpectator {
    fn on_match_complete(&mut self, result: &MatchResult);
    fn on_tournament_complete(&mut self, standings: &ScoreBoard);
}
```

Enables ML training loops, statistical analysis, or real-time visualization.

### Non-Transitive Dominance

The default rules create a dominance cycle:

```
Pos > Neg > Zero > Pos > ...
```

No dominant strategy exists — this is a mixed-strategy Nash equilibrium. Optimal play is random (1/3 each), yielding expected score 0 for both players.

## Quick Start

```rust
use ternary_arena::*;

let alpha = Agent::new(1, "Alpha", vec![Trit::Pos, Trit::Neg, Trit::Zero]);
let beta = Agent::new(2, "Beta", vec![Trit::Zero, Trit::Pos, Trit::Neg]);

let rules = ArenaRules::new();
let result = rules.play_match(&alpha, &beta, 10);
println!("Alpha: {}, Beta: {}", result.score_a, result.score_b);
```

## API

| Type | Description |
|------|-------------|
| `Agent` | id, name, strategy (Vec<Trit>) |
| `ArenaRules` | Payoff matrix and match resolution |
| `MatchResult` | Scores, move history, winner |
| `Tournament` | Round-robin or elimination structure |
| `ScoreBoard` | Cumulative scores with W/L/D |
| `ArenaSpectator` | Observer trait for ML/analysis |

## Architecture Notes

Ternary Arena provides the evaluation framework for ternary strategies in SuperInstance. In γ + η = C, the arena measures both γ (growth — winning strategies accumulate positive scores) and η (avoidance — losing strategies accumulate negative scores, driving selection pressure). Integrates with `ternary-benchmark` for performance measurement and `strategy-ecology` for population dynamics.

See [ARCHITECTURE.md](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md) for game-theoretic architecture.

## References

1. von Neumann, J. & Morgenstern, O. (1944). *Theory of Games and Economic Behavior*. Princeton University Press.
2. Nash, J. (1951). "Non-Cooperative Games." *Annals of Mathematics*, 54(2), 286–295.
3. Axelrod, R. (1984). *The Evolution of Cooperation*. Basic Books.

## License

MIT
