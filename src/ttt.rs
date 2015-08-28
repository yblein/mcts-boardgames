/// Implementation of a Tic-tac-toe game

extern crate mcts;

mod utils;
mod app;

use std::fmt::{Display, Debug, Formatter, Result};

use mcts::{Player, TwoPlayerGame};
use app::Input;
use utils::{Coords2D, draw_board};

const WIDTH: usize = 3;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Token {
	player: Player,
}

impl Display for Token {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match *self {
			Token { player: Player::White } => write!(f, "⛂"),
			Token { player: Player::Black } => write!(f, "⛀"),
		}
	}
}

#[derive(Clone, Default)]
pub struct Move(Coords2D);

impl Debug for Move {
	fn fmt(&self, f: &mut Formatter) -> Result {
		let Move(pos) = *self;
		write!(f, "{}", pos)
	}
}

impl Display for Move {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{:?}", self)
	}
}

impl mcts::Move for Move {}

impl Input for Move {
	fn choose_stdin(moves: &Vec<Move>) -> Move {
		loop {
			let pos = Coords2D::read("Coordinates to play? (e.g., a1)");

			for m in moves {
				if m.0 == pos {
					return m.clone();
				}
			}

			println!("Impossible move");
		}
	}
}

#[derive(Clone)]
pub struct Board([[Option<Token>; WIDTH]; WIDTH]);

impl Board {
	pub fn new() -> Board {
		Board([[None; WIDTH]; WIDTH])
	}
}

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

impl TwoPlayerGame<Move> for Board {
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

	fn play(&mut self, p: Player, m: &Move) {
		let Move(pos) = *m;
		self[pos] = Some(Token { player: p });
	}

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
}

impl Display for Board {
	fn fmt(&self, f: &mut Formatter) -> Result {
		let b: Vec<&[Option<Token>]> = self.0.iter().map(|r| &r[..]).collect();
		draw_board(f, &b[..])
	}
}

fn main() {
	app::App::new(Board::new()).run();
}
