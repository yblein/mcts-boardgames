# MCTS Board Games

This is a generic Rust implementation of the Monte Carlo Tree Search (MCTS) algorithm for two-player board games.
More precisely, the algorithm implemented is UCT (MCTS + UCB1 formula).

There is also a generic framework to make the games actually playable in the terminal.
Several board games are featured as examples (international checkers, Connect Four, ...).
You can find a quick tutorial explaining how to implement a complete game in no-time [here](doc/example_impl.md).

```
   +---+---+---+
 3 |   | ⛀ |   |
   +---+---+---+
 2 |   | ⛂ |   |
   +---+---+---+
 1 |   | ⛂ | ⛀ |
   +---+---+---+
     a   b   c

White turn
Possible moves: a1 a2 c2 a3 c3
Coordinates to play? (e.g., a1)
> a2
```

The AI is not crazy strong at the moment, even when giving it more time to "think".
Suggestions for improvement are welcomed.

## Building Instructions
You need Rust stable and Cargo.


Clone this repository:
```
git clone https://github.com/yblein/mcts-boardgames.git
```

Build all the games provided:
```
cargo build
```

Run a specific game, e.g., checkers:
```
cargo run --bin checkers
```
