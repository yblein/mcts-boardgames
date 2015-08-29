# Example of Game Implementation: Tic-tac-toe

This document illustrates how to implement a game within this framework.
We will take the game of Tic-tac-toe as an example.
The full implementation is [here](src/ttt.rs).

## First Part: Game Mechanics and Automatic AI
The core of the framework provides an AI for any game, provided a simple implementation of the game mechanics.
This section illustrates how to make this implementation for Tic-tac-toe.

First of all, we need a header containing the required imports and a constant defining the board size:
```rust
extern crate mcts;

mod utils;
mod app;

use std::fmt::{Display, Debug, Formatter, Result};

use mcts::{Player, TwoPlayerGame};
use app::Input;
use utils::{Coords2D, draw_board};

const WIDTH: usize = 3;
```

The smallest piece of the game is a token.
A token is represented by a structure with a single field corresponding to the owner of the token:
```rust
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Token {
	player: Player,
}
```

Then we must define a type representing a move in the game.
For convenience, a type `Coords2D` is provided with the framework.
It allows to represent a position in the board.
A move is simply the coordinates where the player wants to play:
```rust
#[derive(Clone, Default)]
pub struct Move(Coords2D);
```
For debugging purposes, `Move` must implement `Debug`.
Here we simply write the underlying value of the move, i.e. its coordinates:
```rust
impl Debug for Move {
	fn fmt(&self, f: &mut Formatter) -> Result {
		let Move(pos) = *self;
		write!(f, "{}", pos)
	}
}
```
We must also explicit the fact that `Move` respect the `Move` trait of the framework:
```rust
impl mcts::Move for Move {}
```

Finally we can define what a game state must look like.
In the case of Tic-tac-toe, a state boils down to the board state.
Thus, the type of the game implementation will be `Board`, which is simply a 2D array of optional tokens:
```rust
#[derive(Clone)]
pub struct Board([[Option<Token>; WIDTH]; WIDTH]);

impl Board {
	pub fn new() -> Board {
		Board([[None; WIDTH]; WIDTH])
	}
}
```

For convenience, we also provide a way to index the board directly with 2D coordinates:
```rust
impl std::ops::Index<Coords2D> for Board {
	type Output = Option<Token>;

	fn index<'a>(&'a self, c: Coords2D) -> &'a Option<Token> {
		&self.0[c.y][c.x]
	}
}

impl std::ops::IndexMut<Coords2D> for Board {
	fn index_mut<'a>(&'a mut self, c: Coords2D) -> &'a mut Option<Token> {
		&mut self.0[c.y][c.x]
	}
}
```

Finally, comes the implementation of the game mechanics:
```rust
impl TwoPlayerGame<Move> for Board {
```
Playing a move is trivial:
```rust
	fn play(&mut self, p: Player, m: &Move) {
		let Move(pos) = *m;
		self[pos] = Some(Token { player: p });
	}
```
The function `possible_moves_in` must return all the possible moves in the vector `moves`, except if the game is over:
```rust
	fn possible_moves_in(&self, _p: Player, moves: &mut Vec<Move>) {
		if self.winner().is_some() {
			return;
		}

		for y in 0..self.0.len() {
			for x in 0..self.0.len() {
				if self.0[y][x].is_none() {
					moves.push(Move(Coords2D { x: x, y: y }))
				}
			}
		}
	}
```
The last function to implement must return the winner of the game, if any.
Here is a basic implementation where all the valid combinations are hardcoded and checked for:
```rust
	fn winner(&self) -> Option<Player> {
		static LIST: [[Coords2D; 3]; 8] = [
			[Coords2D { x: 0, y: 0 }, Coords2D { x: 1, y: 0 }, Coords2D { x: 2, y: 0 }],
			[Coords2D { x: 0, y: 1 }, Coords2D { x: 1, y: 1 }, Coords2D { x: 2, y: 1 }],
			[Coords2D { x: 0, y: 2 }, Coords2D { x: 1, y: 2 }, Coords2D { x: 2, y: 2 }],
			[Coords2D { x: 0, y: 0 }, Coords2D { x: 0, y: 1 }, Coords2D { x: 0, y: 2 }],
			[Coords2D { x: 1, y: 0 }, Coords2D { x: 1, y: 1 }, Coords2D { x: 1, y: 2 }],
			[Coords2D { x: 2, y: 0 }, Coords2D { x: 2, y: 1 }, Coords2D { x: 2, y: 2 }],
			[Coords2D { x: 0, y: 0 }, Coords2D { x: 1, y: 1 }, Coords2D { x: 2, y: 2 }],
			[Coords2D { x: 0, y: 2 }, Coords2D { x: 1, y: 1 }, Coords2D { x: 2, y: 0 }],
		];

		for v in LIST.iter() {
			if self[v[0]] != None && v[1..].iter().all(|c| self[*c] == self[v[0]]) {
				return Some(self[v[0]].unwrap().player);
			}
		}

		return None;
	}
```

To sum up, we defined:
- What is token
- What is move
- What is a game state (just a board here)
- The game mechanics:
	- Playing a move
	- Deciding the winner
	- Computing all the possible moves

And that's all!
The AI is now able to play this game: given a game state, it decides which next best move is the best.

## Second Part: Making it a Complete Application
For now the computer can decide where it should play.
However it is not actually playing complete games, neither is the user.
Allowing the game to be played on the terminal is very easy.
It boils down to printing the game and reading where the player wants to play.

The smallest piece to be printed is a token.
It simply prints a symbol according to the color of its player:
```rust
impl Display for Token {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match *self {
			Token { player: Player::White } => write!(f, "⛂"),
			Token { player: Player::Black } => write!(f, "⛀"),
		}
	}
}
```
Printing of a move is already done in its `Debug` implementation and can be re-used:
```rust
impl Display for Move {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{:?}", self)
	}
}
```
For a move to be readable, it must implement the `Input` trait.
Reading a move consists in reading the coordinates where the player wants to play and returning the corresponding move, if any:
```rust
impl Input for Move {
	fn choose_stdin(moves: &Vec<Move>) -> Move {
		loop {
			let pos = Coords2D::read("Coords2D to play? (e.g., a1)");

			for m in moves {
				if m.0 == pos {
					return m.clone();
				}
			}

			println!("Impossible move");
		}
	}
}
```
Obviously, it is also necessary to implement `Display` for the board.
The framework provides a function `draw_board` which prints a pretty 2D grid with its coordinates.
However it takes as argument a slice of slices and we need to cast the board:
```rust
impl Display for Board {
	fn fmt(&self, f: &mut Formatter) -> Result {
		let b: Vec<&[Option<Token>]> = self.0.iter().map(|r| &r[..]).collect();
		draw_board(f, &b[..])
	}
}
```

We have implemented all the required pieces, we just need a main to actually run the game:
```rust
fn main() {
	app::App::new(Board::new()).run();
}
```

In the end we simply defined a main function and how to:
- print a token
- print a move
- print the board
- read a move

And that's it!
It is now possible to play Tic-tac-toe in the terminal.
The application will ask for each player if it should be a human or an AI and then it will play the game turn by turn:
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
