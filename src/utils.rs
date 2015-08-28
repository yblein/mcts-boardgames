use std;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result;

use std::io::Write;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub struct Coords2D {
	pub x: usize,
	pub y: usize,
}

impl Display for Coords2D {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> Result {
		let letter = std::char::from_u32(97 + self.x as u32).unwrap();
		let digit = self.y + 1;
		write!(f, "{}{}", letter, digit)
	}
}

impl Coords2D {
	pub fn read(msg: &str) -> Coords2D {
		loop {
			print!("{}\n> ", msg);
			std::io::stdout().flush().unwrap();

			let mut input = String::new();
			if std::io::stdin().read_line(&mut input).is_err() {
				continue;
			}
			// TODO: err check

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

			/*
			if x >= $n || y >= $n {
				continue;
			}
			*/

			return Coords2D { x: x, y: y };
		}
	}
}

pub fn draw_board<T: Display>(f: &mut Formatter, board: &[&[Option<T>]]) -> Result {
	let print_hor_line = |f: &mut Formatter| -> Result {
		try!(write!(f, "   "));
		for _ in 0..board.len(){
			try!(write!(f, "+---"));
		}
		write!(f, "+\n")
	};

	try!(write!(f, "\n"));
	try!(print_hor_line(f));

	for (i, row) in board.iter().rev().enumerate() {
		try!(write!(f, "{:>2} | ", board.len() - i));
		for cell in row.iter() {
			try!(match *cell {
				Some(ref t) => write!(f, "{}", t),
				None => write!(f, " "),
			});
			try!(write!(f, " | "));
		}
		try!(write!(f, "\n"));

		try!(print_hor_line(f));
	}

	try!(write!(f, "     "));
	for i in 0..board.len() {
		try!(write!(f, "{}   ", std::char::from_u32(97 + i as u32).unwrap()));
	}
	write!(f, "\n\n")
}
