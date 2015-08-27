extern crate rand;
extern crate mcts;

use std;
use std::io::Write;

use mcts::Player;
use mcts::TwoPlayerBoard;
use mcts::TwoPlayerGame;

use app::AppPlayer;

const WIDTH: usize = 3;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
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

#[derive(Clone)]
pub struct Board ([[Option<Token>; WIDTH]; WIDTH]);

impl Board {
	pub fn new() -> Board {
		Board ( [[None; WIDTH]; WIDTH] )
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

		for y in 0..WIDTH {
			for x in 0..WIDTH {
				if self.0[y][x].is_none() {
					moves.push(Move(Coords { x: x, y: y }))
				}
			}
		}
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

		for (i, row) in self.0.iter().rev().enumerate() {
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
		&self.0[c.y][c.x]
	}
}

impl std::ops::IndexMut<Coords> for Board {
	fn index_mut<'a>(&'a mut self, c: Coords) -> &'a mut Option<Token> {
		&mut self.0[c.y][c.x]
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
			println!("Coords to play? (e.g., a1)");
			let pos = read_coords();

			for m in &moves {
				if m.0 == pos {
					return m.clone();
				}
			}

			println!("Impossible move");
		}
	}
}
