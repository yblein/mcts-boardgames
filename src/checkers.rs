extern crate rand;
extern crate mcts;

use std;
use std::io::Write;

use mcts::Player;
use mcts::TwoPlayerBoard;
use mcts::TwoPlayerGame;

use app::AppPlayer;

const WIDTH: usize = 8;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Coords {
	x: usize,
	y: usize,
}

impl std::fmt::Display for Coords {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let letter = std::char::from_u32(97 + self.x as u32).unwrap();
		let digit = self.y + 1;
		write!(f, "{}{}", letter, digit)
	}
}

#[derive(Debug, Clone)]
pub struct Board {
	board: [[Option<Token>; WIDTH]; WIDTH],
	nb_crowned_moves_without_capture: usize,
}

impl Board {
	pub fn new() -> Board {
		let mut b = Board {
			board: [[None; WIDTH]; WIDTH],
			nb_crowned_moves_without_capture: 0,
		};

		// FIXME: to use when stable
		//for i in (0..WIDTH).step_by(2) {

		for row in 0..(WIDTH / 2 - 1) {
			for col in 0..WIDTH {
				if col % 2 == 1 {
					continue
				}
				let base = if row % 2 == 1 { 1 } else { 0 };
				if base + col < WIDTH {
					b.board[row][base + col] = Some(Token { player: Player::White, crowned: false });
				}
			}
		}

		for row in (WIDTH / 2 + 1)..WIDTH {
			for col in 0..WIDTH {
				if col % 2 == 1 {
					continue
				}
				let base = if row % 2 == 1 { 1 } else { 0 };
				if base + col < WIDTH {
					b.board[row][base + col] = Some(Token { player: Player::Black, crowned: false });
				}
			}
		}

		return b;
	}

	fn is_in(&self, x: isize, y: isize) -> bool {
		x >= 0 && x < WIDTH as isize && y >= 0 && y < WIDTH as isize
	}

	fn possible_moves_from(&self, src: Coords, moves: &mut Vec<Move>) {
		let x = src.x as isize;
		let y = src.y as isize;
		let t = self[src].unwrap();

		let mut dy = if t.player == Player::White && !t.crowned { 1 } else { -1 };

		for _ in 0..2 {
			let mut dx = -1;

			for _ in 0..2 {
				let mut cx = x + dx;
				let mut cy = y + dy;

				while self.is_in(cx, cy) {
					let dst = Coords { x: cx as usize, y: cy as usize };
					match self[dst] {
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
		let t = self[src].unwrap();

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
							match self[c] {
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
							match self[c] {
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
		if self[m.src].unwrap().crowned && m.captured.is_empty() {
			self.nb_crowned_moves_without_capture += 1;
		} else {
			self.nb_crowned_moves_without_capture = 0;
		}

		// perform the move
		self[m.dst] = self[m.src];
		self[m.src] = None;

		// remove captured tokens
		for c in m.captured.iter() {
			self[*c] = None;
		}

		// crown the piece if necessary
		if ((p == Player::White && m.dst.y == WIDTH - 1) || (p == Player::Black && m.dst.y == 0))
				&& !self[m.dst].unwrap().crowned {
			self[m.dst].as_mut().unwrap().crowned = true;
		}
	}

	fn possible_moves_in(&self, p: Player, moves: &mut Vec<Move>) {
		if self.nb_crowned_moves_without_capture >= 25 {
			return;
		}

		for y in 0..WIDTH {
			for x in 0..WIDTH {
				match self.board[y][x] {
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
		fn print_hor_line(f: &mut std::fmt::Formatter) -> std::fmt::Result {
			try!(write!(f, "   "));
			for _ in 0..WIDTH {
				try!(write!(f, "+---"));
			}
			write!(f, "+\n")
		}

		try!(write!(f, "\n"));
		try!(print_hor_line(f));

		for (i, row) in self.board.iter().rev().enumerate() {
			try!(write!(f, "{:>2} | ", WIDTH - i));
			for cell in row.iter() {
				try!(match *cell {
					Some(t) => write!(f, "{}", t),
					None => write!(f, " "),
				});
				try!(write!(f, " | "));
			}
			try!(write!(f, "\n"));

			try!(print_hor_line(f));
		}

		try!(write!(f, "     "));
		for i in 0..WIDTH {
			try!(write!(f, "{}   ", std::char::from_u32(97 + i as u32).unwrap()));
		}
		write!(f, "\n\n")
	}
}

impl std::ops::Index<Coords> for Board {
	type Output = Option<Token>;

	fn index<'a>(&'a self, c: Coords) -> &'a Option<Token> {
		&self.board[c.y][c.x]
	}
}

impl std::ops::IndexMut<Coords> for Board {
	fn index_mut<'a>(&'a mut self, c: Coords) -> &'a mut Option<Token> {
		&mut self.board[c.y][c.x]
	}
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
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

pub struct HumanPlayer;

impl AppPlayer<Board, Move> for HumanPlayer {
	fn get_next_move(&mut self, game: &TwoPlayerGame<Board, Move>) -> Move {
		fn read_coords() -> Coords {
			loop {
				print!("> ");
				std::io::stdout().flush().unwrap();

				let mut input = String::new();
				if std::io::stdin().read_line(&mut input).is_err() {
					continue;
				}

				let s: &str = input.trim();
				let tmp: Vec<char> = s[0..1].to_uppercase().chars().collect();
				let letter: char = tmp[0];
				let number: usize = if let Some(n) = s[1..].parse::<usize>().ok() {
					n
				} else {
					continue;
				};

				let x = letter as usize - 'A' as usize;
				let y = number as usize - 1;

				if x >= WIDTH || y >= WIDTH {
					continue;
				}

				return Coords { x: x, y: y };
			}
		}

		println!("Possible moves:");
		let moves = game.possible_moves();
		for m in &moves {
			println!("{}", m)
		}

		loop {
			println!("Token to move? (e.g., a1)");
			let src = read_coords();
			let mut moves_from_src: Vec<Move> = moves.iter().filter(|ref m| m.src == src).cloned().collect();

			match moves_from_src.len() {
				0 => {
					println!("Invalid coordinates");
					continue;
				},
				1 => return moves_from_src[0].clone(),
				_ => (),
			}

			println!("Destination? (e.g., b2)");
			let dst = read_coords();
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
