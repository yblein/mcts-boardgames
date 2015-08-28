extern crate mcts;

use std;
use std::io::Write;

use mcts::Player;
use mcts::TwoPlayerBoard;

use app::Input;

make_types_square_grid_2d!(Grid, Option<Token>, 3);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Token {
	player: Player,
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			Token { player: Player::White } => write!(f, "⛂"),
			Token { player: Player::Black } => write!(f, "⛀"),
		}
	}
}

#[derive(Clone)]
pub struct Board {
	grid: Grid,
}

impl Board {
	pub fn new() -> Board {
		Board { grid: Grid::empty() }
	}
}

impl TwoPlayerBoard<Move> for Board {
	fn winner(&self) -> Option<Player> {
		static LIST: [[Coords; 3]; 8] = [
			[Coords { x: 0, y: 0 }, Coords { x: 1, y: 0 }, Coords { x: 2, y: 0 }],
			[Coords { x: 0, y: 1 }, Coords { x: 1, y: 1 }, Coords { x: 2, y: 1 }],
			[Coords { x: 0, y: 2 }, Coords { x: 1, y: 2 }, Coords { x: 2, y: 2 }],
			[Coords { x: 0, y: 0 }, Coords { x: 0, y: 1 }, Coords { x: 0, y: 2 }],
			[Coords { x: 1, y: 0 }, Coords { x: 1, y: 1 }, Coords { x: 1, y: 2 }],
			[Coords { x: 2, y: 0 }, Coords { x: 2, y: 1 }, Coords { x: 2, y: 2 }],
			[Coords { x: 0, y: 0 }, Coords { x: 1, y: 1 }, Coords { x: 2, y: 2 }],
			[Coords { x: 0, y: 2 }, Coords { x: 1, y: 1 }, Coords { x: 2, y: 0 }],
		];

		for v in LIST.iter() {
			if self.grid[v[0]] != None && v[1..].iter().all(|c| self.grid[*c] == self.grid[v[0]]) {
				return Some(self.grid[v[0]].unwrap().player);
			}
		}

		return None;
	}

	fn play(&mut self, p: Player, m: &Move) {
		let Move(pos) = *m;
		self.grid[pos] = Some(Token { player: p });
	}

	fn possible_moves_in(&self, _p: Player, moves: &mut Vec<Move>) {
		if self.winner().is_some() {
			return;
		}

		for y in 0..self.grid.0.len() {
			for x in 0..self.grid.0.len() {
				if self.grid.0[y][x].is_none() {
					moves.push(Move(Coords { x: x, y: y }))
				}
			}
		}
	}
}

impl std::fmt::Display for Board {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.grid)
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Move(Coords);

impl std::fmt::Display for Move {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let Move(pos) = *self;
		write!(f, "{}", pos)
	}
}

impl mcts::Move for Move {}

impl Input for Move {
	fn choose_stdin(moves: &Vec<Move>) -> Move {
		loop {
			let pos = Coords::read("Coords to play? (e.g., a1)");

			for m in moves {
				if m.0 == pos {
					return m.clone();
				}
			}

			println!("Impossible move");
		}
	}
}
