macro_rules! make_type_square_grid_2d {
	($name:ident, $t:ty, $n:expr) => {
		#[derive(Clone)]
		struct $name([[$t; $n]; $n]);

		impl $name {
			fn empty() -> $name {
				$name([[Default::default(); $n]; $n])
			}
		}

		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				fn print_hor_line(f: &mut std::fmt::Formatter) -> std::fmt::Result {
					try!(write!(f, "   "));
					for _ in 0..$n {
						try!(write!(f, "+---"));
					}
					write!(f, "+\n")
				}

				try!(write!(f, "\n"));
				try!(print_hor_line(f));

				for (i, row) in self.0.iter().rev().enumerate() {
					try!(write!(f, "{:>2} | ", $n - i));
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
				for i in 0..$n {
					try!(write!(f, "{}   ", std::char::from_u32(97 + i as u32).unwrap()));
				}
				write!(f, "\n\n")
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

		impl std::ops::Index<Coords> for $name {
			type Output = $t;

			fn index<'a>(&'a self, c: Coords) -> &'a $t {
				&self.0[c.y][c.x]
			}
		}

		impl std::ops::IndexMut<Coords> for $name {
			fn index_mut<'a>(&'a mut self, c: Coords) -> &'a mut $t {
				&mut self.0[c.y][c.x]
			}
		}

		fn read_coords(msg: &str) -> Coords {
			loop {
				print!("{}\n> ", msg);
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

				if x >= $n || y >= $n {
					continue;
				}

				return Coords { x: x, y: y };
			}
		}
	}
}
