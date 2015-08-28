extern crate mcts;

use std;
use std::io::Write;

use mcts::Player;
use mcts::TwoPlayerBoard;

use app::Input;

make_types_square_grid_2d!(Grid, Option<Token>, 8);

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Token {
	player: Player,
	crowned: bool,
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			Token { player: Player::White, crowned: false } => write!(f, "⛂"),
			Token { player: Player::White, crowned: true  } => write!(f, "⛃"),
			Token { player: Player::Black, crowned: false } => write!(f, "⛀"),
			Token { player: Player::Black, crowned: true  } => write!(f, "⛁"),
		}
	}
}

#[derive(Clone)]
pub struct Board {
	grid: Grid,
	nb_crowned_moves_without_capture: usize,
}

impl Board {
	pub fn new() -> Board {
		let mut b = Board {
			grid: Grid::empty(),
			nb_crowned_moves_without_capture: 0,
		};

		// FIXME: to use when stable
		//for col in (0..b.grid.0.len()).step_by(2) {

		for row in 0..(b.grid.0.len() / 2 - 1) {
			for col in 0..b.grid.0.len() {
				if col % 2 == 1 {
					continue
				}
				let base = if row % 2 == 1 { 1 } else { 0 };
				if base + col < b.grid.0.len() {
					b.grid.0[row][base + col] = Some(Token { player: Player::White, crowned: false });
				}
			}
		}

		for row in (b.grid.0.len() / 2 + 1)..b.grid.0.len() {
			for col in 0..b.grid.0.len() {
				if col % 2 == 1 {
					continue
				}
				let base = if row % 2 == 1 { 1 } else { 0 };
				if base + col < b.grid.0.len() {
					b.grid.0[row][base + col] = Some(Token { player: Player::Black, crowned: false });
				}
			}
		}

		return b;
	}

	fn is_in(&self, x: isize, y: isize) -> bool {
		x >= 0 && x < self.grid.0.len() as isize && y >= 0 && y < self.grid.0.len() as isize
	}

	fn possible_moves_from(&self, src: Coords, moves: &mut Vec<Move>) {
		let x = src.x as isize;
		let y = src.y as isize;
		let t = self.grid[src].unwrap();

		let mut dy = if t.player == Player::White && !t.crowned { 1 } else { -1 };

		for _ in 0..2 {
			let mut dx = -1;

			for _ in 0..2 {
				let mut cx = x + dx;
				let mut cy = y + dy;

				while self.is_in(cx, cy) {
					let dst = Coords { x: cx as usize, y: cy as usize };
					match self.grid[dst] {
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
		let mut captured: Vec<Coords> = Vec::new();
		self.possible_captures(src, src, &mut captured, moves);
	}

	fn possible_captures(&self, src: Coords, pos: Coords, captured: &mut Vec<Coords>, moves: &mut Vec<Move>) {
		let x = pos.x as isize;
		let y = pos.y as isize;
		let t = self.grid[src].unwrap();

		let mut done = true;
		let mut dy = -1;

		for _ in 0..2 {
			let mut dx = -1;

			for _ in 0..2 {
				let mut cx = x + dx;
				let mut cy = y + dy;
				let mut capture_target = None;

				while self.is_in(cx, cy) {
					let c = Coords { x: cx as usize, y: cy as usize };

					match capture_target {
						None => {
							match self.grid[c] {
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
							match self.grid[c] {
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

impl TwoPlayerBoard<Move> for Board {
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
		if self.grid[m.src].unwrap().crowned && m.captured.is_empty() {
			self.nb_crowned_moves_without_capture += 1;
		} else {
			self.nb_crowned_moves_without_capture = 0;
		}

		// perform the move
		self.grid[m.dst] = self.grid[m.src];
		self.grid[m.src] = None;

		// remove captured tokens
		for c in m.captured.iter() {
			self.grid[*c] = None;
		}

		// crown the piece if necessary
		if ((p == Player::White && m.dst.y == self.grid.0.len() - 1) || (p == Player::Black && m.dst.y == 0))
				&& !self.grid[m.dst].unwrap().crowned {
			self.grid[m.dst].as_mut().unwrap().crowned = true;
		}
	}

	fn possible_moves_in(&self, p: Player, moves: &mut Vec<Move>) {
		if self.nb_crowned_moves_without_capture >= 25 {
			return;
		}

		// TODO: check moves with capture first
		// if there exist such moves, then there is no need to check moves without capture

		for y in 0..self.grid.0.len() {
			for x in 0..self.grid.0.len() {
				match self.grid.0[y][x] {
					Some(Token { player: p2, crowned: _ }) if p2 == p => {
						let c = Coords { x: x, y: y };
						self.possible_moves_from(c, moves);
					}
					_ => (),
				}
			}
		}

		let max_captures = moves.iter().fold(0, |max, m| std::cmp::max(max, m.captured.len()));
		moves.retain(|a| a.captured.len() == max_captures);
	}
}

impl std::fmt::Display for Board {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.grid)
	}
}

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Move {
	src: Coords,
	dst: Coords,
	captured: Vec<Coords>,
}

impl std::fmt::Display for Move {
	fn fmt(&self, f:&mut std::fmt::Formatter) -> std::fmt::Result {
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
			let src = Coords::read("Token to move? (e.g., a1)");
			let mut moves_from_src: Vec<Move> = moves.iter().filter(|ref m| m.src == src).cloned().collect();

			match moves_from_src.len() {
				0 => {
					println!("Invalid coordinates");
					continue;
				},
				1 => return moves_from_src[0].clone(),
				_ => (),
			}

			let dst = Coords::read("Destination? (e.g., b2)");
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
