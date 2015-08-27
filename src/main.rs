extern crate rand;
extern crate mcts;

use std::io::Write;

mod checkers;
mod app;

fn main() {
	fn make_player(msg: &str) -> Box<app::AppPlayer<checkers::Board, checkers::Move>> {
		println!("{}?", msg);
		println!("0: Player");
		println!("1: Computer");
		print!("> ");
		std::io::stdout().flush().unwrap();

		let mut s = String::new();
		std::io::stdin().read_line(&mut s).unwrap();

		let mode = match s.trim_right().parse::<usize>() {
			Ok(n) if n <= 1 => n,
			_ => panic!("Invalid number"),
		};

		match mode {
			0 => Box::new(checkers::HumanPlayer),
			_ => Box::new(app::ComputerPlayer::new(rand::thread_rng(), 1000, 1.0)),
		}
	}

	let mut app = app::App::new(
        checkers::Board::new(),
		make_player("White player"),
		make_player("Black player"),
	);

	app.run();
}
