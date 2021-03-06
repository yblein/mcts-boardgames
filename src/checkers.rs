/// Implementation of a checkers (or draughts) with international rules

extern crate mcts;

mod utils;
mod app;

use std::fmt::{Display, Debug, Formatter, Result};

use mcts::{Player, TwoPlayerGame};
use app::Input;
use utils::{Coords2D, draw_board};

const WIDTH: usize = 8;

#[derive(Clone)]
struct Board([[Option<Token>; WIDTH]; WIDTH]);

impl Board {
	fn empty() -> Board {
		Board([[Default::default(); WIDTH]; WIDTH])
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

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Token {
	player: Player,
	crowned: bool,
}

impl Display for Token {
	fn fmt(&self, f: &mut Formatter) -> Result {
		match *self {
			Token { player: Player::White, crowned: false } => write!(f, "⛂"),
			Token { player: Player::White, crowned: true  } => write!(f, "⛃"),
			Token { player: Player::Black, crowned: false } => write!(f, "⛀"),
			Token { player: Player::Black, crowned: true  } => write!(f, "⛁"),
		}
	}
}

#[derive(Clone)]
pub struct Game {
	board: Board,
	nb_crowned_moves_without_capture: usize,
}

impl Game {
	pub fn new() -> Game {
		let mut g = Game {
			board: Board::empty(),
			nb_crowned_moves_without_capture: 0,
		};

		// FIXME: to use when stable
		//for col in (0..g.board.0.len()).step_by(2) {

		for row in 0..(g.board.0.len() / 2 - 1) {
			for col in 0..g.board.0.len() {
				if col % 2 == 1 {
					continue
				}
				let base = if row % 2 == 1 { 1 } else { 0 };
				if base + col < g.board.0.len() {
					g.board.0[row][base + col] = Some(Token { player: Player::White, crowned: false });
				}
			}
		}

		for row in (g.board.0.len() / 2 + 1)..g.board.0.len() {
			for col in 0..g.board.0.len() {
				if col % 2 == 1 {
					continue
				}
				let base = if row % 2 == 1 { 1 } else { 0 };
				if base + col < g.board.0.len() {
					g.board.0[row][base + col] = Some(Token { player: Player::Black, crowned: false });
				}
			}
		}

		return g;
	}

	fn is_in(&self, x: isize, y: isize) -> bool {
		x >= 0 && x < self.board.0.len() as isize && y >= 0 && y < self.board.0.len() as isize
	}

	fn possible_moves_from(&self, src: Coords2D, moves: &mut Vec<Move>) {
		let x = src.x as isize;
		let y = src.y as isize;
		let t = self.board[src].unwrap();

		let mut dy = if t.player == Player::White && !t.crowned { 1 } else { -1 };

		for _ in 0..2 {
			let mut dx = -1;

			for _ in 0..2 {
				let mut cx = x + dx;
				let mut cy = y + dy;

				while self.is_in(cx, cy) {
					let dst = Coords2D { x: cx as usize, y: cy as usize };
					match self.board[dst] {
						None => moves.push(Move {
							src: src,
							dst: dst,
							captured: Vec::new(),
						}),
						_ => break,
					}

					if !t.crowned {
						break
					}

					cx += dx;
					cy += dy;
				}

				dx = -dx;
			}
			
			if !t.crowned {
				break;
			}

			dy = -dy;
		}

		// plus moves with captures
		let mut captured: Vec<Coords2D> = Vec::new();
		self.possible_captures(src, src, &mut captured, moves);
	}

	fn possible_captures(&self, src: Coords2D, pos: Coords2D, captured: &mut Vec<Coords2D>, moves: &mut Vec<Move>) {
		let x = pos.x as isize;
		let y = pos.y as isize;
		let t = self.board[src].unwrap();

		let mut done = true;
		let mut dy = -1;

		for _ in 0..2 {
			let mut dx = -1;

			for _ in 0..2 {
				let mut cx = x + dx;
				let mut cy = y + dy;
				let mut capture_target = None;

				while self.is_in(cx, cy) {
					let c = Coords2D { x: cx as usize, y: cy as usize };

					match capture_target {
						None => {
							match self.board[c] {
								Some(Token { player: p, crowned: _ }) => {
									// can't go over:
									// - its own tokens
									// - the same token twice in a move
									if p == t.player || captured.contains(&c) {
										break;
									}
									capture_target = Some(c);
								},
								_ => if !t.crowned {
									break;
								},
							}
						},
						Some(cap) => {
							match self.board[c] {
								None => {
									captured.push(cap);
									self.possible_captures(src, c, captured, moves);
									captured.pop();

									done = false;

									if !t.crowned {
										break;
									}
								}
								_ => break,
							}
						}
					}

					cx += dx;
					cy += dy;
				}

				dx = -dx;
			}

			dy = -dy;
		}

		if done && src != pos {
			moves.push(Move {
				src: src,
				dst: pos,
				captured: captured.clone()
			});
		}
	}
}

impl TwoPlayerGame<Move> for Game {
	fn winner(&self) -> Option<Player> {
		match (self.possible_moves(Player::White).is_empty(), self.possible_moves(Player::Black).is_empty()) {
			(true, true) => None,
			(true, false) => Some(Player::Black),
			(false, true) => Some(Player::White),
			_ => panic!("winner() must be called on a finished game"),
		}
	}

	fn play(&mut self, p: Player, m: &Move) {
		// update the number of crowned move without capture
		if self.board[m.src].unwrap().crowned && m.captured.is_empty() {
			self.nb_crowned_moves_without_capture += 1;
		} else {
			self.nb_crowned_moves_without_capture = 0;
		}

		// perform the move
		self.board[m.dst] = self.board[m.src];
		self.board[m.src] = None;

		// remove captured tokens
		for c in m.captured.iter() {
			self.board[*c] = None;
		}

		// crown the piece if necessary
		if ((p == Player::White && m.dst.y == self.board.0.len() - 1) || (p == Player::Black && m.dst.y == 0))
				&& !self.board[m.dst].unwrap().crowned {
			self.board[m.dst].as_mut().unwrap().crowned = true;
		}
	}

	fn possible_moves_in(&self, p: Player, moves: &mut Vec<Move>) {
		if self.nb_crowned_moves_without_capture >= 25 {
			return;
		}

		// TODO: check moves with capture first
		// if there exist such moves, then there is no need to check moves without capture

		for y in 0..self.board.0.len() {
			for x in 0..self.board.0.len() {
				match self.board.0[y][x] {
					Some(Token { player: p2, crowned: _ }) if p2 == p => {
						let c = Coords2D { x: x, y: y };
						self.possible_moves_from(c, moves);
					}
					_ => (),
				}
			}
		}

		// FIXME: use max_by when stable
		let max_captures = moves.iter().fold(0, |max, m| std::cmp::max(max, m.captured.len()));
		moves.retain(|a| a.captured.len() == max_captures);
	}
}

impl Display for Game {
	fn fmt(&self, f: &mut Formatter) -> Result {
		let b: Vec<&[Option<Token>]> = self.board.0.iter().map(|r| &r[..]).collect();
		draw_board(f, &b[..])
	}
}

#[derive(Clone, Default)]
pub struct Move {
	src: Coords2D,
	dst: Coords2D,
	captured: Vec<Coords2D>,
}

impl Debug for Move {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}", self)
	}
}

impl Display for Move {
	fn fmt(&self, f: &mut Formatter) -> Result {
		try!(write!(f, "{} -> {}", self.src, self.dst));

		if !self.captured.is_empty() {
			try!(write!(f, ", with capture of "));
			try!(write!(f, "{}", self.captured[0]));

			for c in &self.captured[1..] {
				try!(write!(f, ", {}", c));
			}
		}

		return Ok(());
	}
}

impl mcts::Move for Move {}

impl Input for Move {
	fn choose_stdin(moves: &Vec<Move>) -> Move {
		loop {
			let src = Coords2D::read("Token to move? (e.g., a1)");
			let mut moves_from_src: Vec<Move> = moves.iter().filter(|ref m| m.src == src).cloned().collect();

			match moves_from_src.len() {
				0 => {
					println!("Invalid coordinates");
					continue;
				},
				1 => return moves_from_src[0].clone(),
				_ => (),
			}

			let dst = Coords2D::read("Destination? (e.g., b2)");
			moves_from_src.retain(|ref m| m.dst == dst);

			match moves_from_src.len() {
				0 => {
					println!("Invalid coordinates");
					continue;
				},
				1 => return moves_from_src[0].clone(),
				_ => (),
			}

			println!("Impossible move");
		}
	}
}

fn main() {
	app::App::new(Game::new()).run();
}
