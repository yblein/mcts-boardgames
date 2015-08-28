extern crate mcts;

mod app;

use std::fmt::Debug;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;
use std::io::Write;

use mcts::Player;
use mcts::TwoPlayerBoard;

use app::Input;

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

#[derive(Clone, Default, PartialEq, Eq)]
pub struct Coords(usize);

impl Coords {
	fn read(msg: &str) -> Coords {
		loop {
			print!("{}\n> ", msg);
			std::io::stdout().flush().unwrap();

			let mut input = String::new();
			if std::io::stdin().read_line(&mut input).is_err() {
				continue;
			}

			let s: &str = input.trim();
			let number: usize = if let Some(n) = s.parse::<usize>().ok() {
				n
			} else {
				continue;
			};

			let x = number as usize - 1;

			if x >= 7 {
				continue;
			}

			return Coords(x);
		}
	}
}

impl Display for Coords {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}", self.0 + 1)
	}
}

impl Debug for Coords {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}", self)
	}
}

#[derive(Clone, Default)]
pub struct Move(Coords);

impl Display for Move {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}", self.0)
	}
}

impl Debug for Move {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}", self)
	}
}

impl mcts::Move for Move {}

impl Input for Move {
	fn choose_stdin(moves: &Vec<Move>) -> Move {
		loop {
			let pos = Coords::read("Column to play? (e.g., a1)");

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
pub struct Grid([[Option<Token>; 6]; 7]);

impl Grid {
	fn empty() -> Grid {
		Grid([[None; 6]; 7])
	}
}

impl Display for Grid {
	fn fmt(&self, f: &mut Formatter) -> Result {
		fn print_hor_line(f: &mut Formatter) -> Result {
			try!(write!(f, " "));
			for _ in 0..7 {
				try!(write!(f, "+---"));
			}
			write!(f, "+\n")
		}

		try!(write!(f, "\n"));
		try!(print_hor_line(f));

		for row in (0..6).rev() {
			try!(write!(f, " | "));
			for col in 0..7 {
				try!(match self.0[col][row] {
					Some(t) => write!(f, "{}", t),
					None => write!(f, " "),
				});
				try!(write!(f, " | "));
			}
			try!(write!(f, "\n"));
			try!(print_hor_line(f));
		}

		try!(write!(f, "   "));
		for i in 0..self.0.len() {
			try!(write!(f, "{}   ", i + 1));
		}
		write!(f, "\n\n")
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
		// horizontally
		for row in 0..6 {
			let mut count = 0;
			let mut last = Player::White;

			for col in 0..7 {
				match self.grid.0[col][row] {
					Some(Token { player: p }) =>
						if p == last {
							count += 1;
							if count >= 4 {
								return Some(last);
							}
						} else {
							last = p;
							count = 1;
						},
					None => count = 0,
				}
			}
		}

		// vertically
		for col in 0..7 {
			let mut count = 0;
			let mut last = Player::White;

			for row in 0..6 {
				match self.grid.0[col][row] {
					Some(Token { player: p }) =>
						if p == last {
							count += 1;
							if count >= 4 {
								return Some(last);
							}
						} else {
							last = p;
							count = 1;
						},
					None => count = 0,
				}
			}
		}

		// ascending diagonal
		for col in 0..4 {
			for row in 0..3 {
				let mut count = 0;
				let mut last = Player::White;
				let mut offset = 0;

				while col + offset < 7 && row + offset < 6 {
					match self.grid.0[col + offset][row + offset] {
						Some(Token { player: p }) =>
							if p == last {
								count += 1;
								if count >= 4 {
									return Some(last);
								}
							} else {
								last = p;
								count = 1;
							},
						None => count = 0,
					}

					offset += 1;
				}
			}
		}

		// descending diagonal
		for col in 0..4 {
			for row in 0..3 {
				let mut count = 0;
				let mut last = Player::White;
				let mut offset = 0;

				while col + offset < 7 && row + offset < 6 {
					match self.grid.0[col + offset][5 - (row + offset)] {
						Some(Token { player: p }) =>
							if p == last {
								count += 1;
								if count >= 4 {
									return Some(last);
								}
							} else {
								last = p;
								count = 1;
							},
						None => count = 0,
					}

					offset += 1;
				}
			}
		}
		return None;
	}

	fn play(&mut self, p: Player, m: &Move) {
		let Move(Coords(col)) = *m;

		for cell in self.grid.0[col].iter_mut() {
			if cell.is_none() {
				*cell = Some(Token{player: p});
				return;
			}
		}
	}

	fn possible_moves_in(&self, _p: Player, moves: &mut Vec<Move>) {
		if self.winner().is_some() {
			return;
		}

		for col in 0..self.grid.0.len() {
			if self.grid.0[col][5].is_none() {
				moves.push(Move(Coords(col)));
			}
		}
	}
}

impl Display for Board {
	fn fmt(&self, f: &mut Formatter) -> Result {
		write!(f, "{}", self.grid)
	}
}

fn main() {
	app::App::new(Board::new()).run();
}
