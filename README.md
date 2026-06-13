# ternary-arena

**Multi-agent competition arena for balanced ternary systems.**

`ternary-arena` provides a structured environment where agents carrying ternary strategies (sequences of trit moves from $\{-1, 0, +1\}$) compete in matches governed by ternary game-theoretic rules. It supports single matches, elimination tournaments, scoreboard tracking, and spectator-based observation for analysis.

## Why It Matters

Multi-agent reinforcement learning and evolutionary game theory require controlled environments where strategies can be evaluated against each other under well-defined rules. In ternary systems — where every action is one of three choices — the game space is rich enough to exhibit non-trivial dynamics (rock-paper-scissors-style cycles) yet small enough for exhaustive analysis.

This crate provides that environment. Each agent plays a strategy vector $\mathbf{s} \in \{-1, 0, +1\}^n$, and outcomes are determined by a payoff matrix. The arena supports tournament brackets, leaderboard ranking, and spectator analytics including win-rate computation.

## How It Works

### Ternary Game Rules

The default rule system is a cyclic dominance game on $\{-1, 0, +1\}$, analogous to rock-paper-scissors:

$$\text{Pos}(+1) \succ \text{Neg}(-1) \succ \text{Zero}(0) \succ \text{Pos}(+1)$$

The payoff matrix $M$ assigns points for each $(a, b)$ pair:

$$M(a, b) = \begin{cases} 3 & \text{if } a \text{ dominates } b \\ 1 & \text{if } a = b \text{ (draw)} \\ 0 & \text{if } b \text{ dominates } a \end{cases}$$

Each round resolves to $(M(a,b),\; M(b,a))$, ensuring a **zero-sum-like** structure where dominance yields 3 points for the winner and 0 for the loser.

### Match Resolution

A match consists of $r$ rounds. In each round $t$, agent $A$ plays move $a_t = s_A[t \bmod |s_A|]$ and agent $B$ plays $b_t = s_B[t \bmod |s_B|]$. Strategies wrap cyclically, enabling arbitrarily long matches from finite strategy vectors.

**Total score:**

$$\text{Score}_A = \sum_{t=0}^{r-1} M(a_t, b_t), \qquad \text{Score}_B = \sum_{t=0}^{r-1} M(b_t, a_t)$$

**Complexity:** $O(r)$ for $r$ rounds. Each round involves two modular lookups ($O(1)$) and one payoff table lookup ($O(1)$).

### Tournament Structure

Tournaments use single-elimination brackets. For $n$ agents:

$$\text{matches} = n - 1 \quad \text{(for power-of-two } n\text{)}$$

When $n$ is odd, the last unpaired agent receives a **bye** (auto-advances). Draws are resolved by tiebreak: the first-listed agent advances.

**Complexity:** $O(n)$ matches total, $O(\log_2 n)$ rounds for power-of-two brackets.

### Scoreboard and Leaderboard

The scoreboard accumulates scores, wins, losses, and draws across multiple matches. The leaderboard sorts agents by cumulative score:

$$\text{rank}(i) < \text{rank}(j) \iff \text{Score}_i > \text{Score}_j$$

**Complexity:** $O(1)$ per match recorded, $O(n \log n)$ for leaderboard sorting.

### Spectator Analytics

Spectators observe matches and compute **win rates**:

$$W_i = \frac{|\{m : \text{winner}(m) = i\}|}{|\{m : i \in m\}|}$$

where the denominator counts matches in which agent $i$ participated. Win rate is undefined ($0.0$) for agents with no observed matches.

### Nash Equilibrium Note

In the default cyclic game, no pure strategy is a Nash equilibrium. The mixed-strategy Nash equilibrium is the uniform distribution $U\{-1, 0, +1\}$, where each action is played with probability $\frac{1}{3}$. Agents with non-uniform strategies are exploitable.

## Quick Start

```toml
[dependencies]
ternary-arena = "0.1"
```

```rust
use ternary_arena::{Agent, Arena, ArenaRules, Match, Tournament, Trit};

// Create agents with strategies
let agent_a = Agent::new(1, "aggressive", vec![Trit::Pos, Trit::Pos, Trit::Neg]);
let agent_b = Agent::new(2, "defensive", vec![Trit::Zero, Trit::Neg, Trit::Zero]);

// Run a single match
let mut arena = Arena::new("colosseum");
let m = arena.play_match(Match::new(agent_a, agent_b, 10));
assert!(m.finished);

// Run a tournament
let agents = vec![
    Agent::new(1, "a", vec![Trit::Pos]),
    Agent::new(2, "b", vec![Trit::Neg]),
    Agent::new(3, "c", vec![Trit::Zero]),
    Agent::new(4, "d", vec![Trit::Pos, Trit::Zero]),
];
let champion = arena.play_tournament(Tournament::new("cup", agents, 5));
println!("Champion: {:?}", champion);

// Check standings
let board = arena.scoreboard.leaderboard();
println!("Leaderboard: {:?}", board);
```

## API

| Type | Purpose | Key Methods |
|------|---------|-------------|
| `Trit` | The $\{-1, 0, +1\}$ value type | `value()`, `from_i8()` |
| `Agent` | Strategy-carrying competitor | `new()`, `move_at()`, `strategy_len()` |
| `ArenaRules` | Payoff matrix and resolution | `resolve()`, `flat()` |
| `Match` | Two-agent game over $r$ rounds | `play()`, `winner()`, `total_points()` |
| `ScoreBoard` | Cumulative scoring | `record()`, `score()`, `wins()`, `leaderboard()` |
| `Tournament` | Single-elimination bracket | `run()`, `match_count()` |
| `ArenaSpectator` | Match observer with analytics | `observe()`, `win_rate()` |
| `Arena` | Top-level competition environment | `play_match()`, `play_tournament()` |

## Architecture Notes

The arena operates within the SuperInstance conservation law **γ + η = C**. Each match converts strategic energy ($\gamma$) into outcome entropy ($\eta$): a decisive result concentrates information (low $\eta$), while a draw maximizes entropy ($\eta \to C$) since no resolution occurs. The cyclic dominance structure ensures that no single strategy dominates indefinitely — the system oscillates around the Nash equilibrium, maintaining the conservation bound.

Tournament brackets progressively concentrate $\gamma$ into the champion, representing the maximum-likelihood best strategy. The elimination process is an information funnel: $n$ agents enter with $n$ strategies, but only one emerges, having accumulated the $\gamma$ of defeated opponents.

## References

- Nash, J. *Equilibrium Points in n-Person Games.* PNAS 1950. — Nash equilibrium in multi-agent games.
- von Neumann, J. & Morgenstern, O. *Theory of Games and Economic Behavior.* Princeton, 1944. — Zero-sum game theory.
- Wolfram, S. *A New Kind of Science.* Wolfram Media, 2002. — Cyclic cellular automata and competitive systems.
- Axelrod, R. *The Evolution of Cooperation.* Basic Books, 1984. — Tournament-style strategy evaluation.

## License

MIT
