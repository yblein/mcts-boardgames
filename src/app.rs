extern crate rand;
extern crate mcts;

use std;
use std::io::Write;
use std::fmt::Display;

use mcts::TwoPlayerGame;
use mcts::TwoPlayerBoard;
use mcts::Move;

pub trait AppPlayer<B: TwoPlayerBoard<M>, M: Move> {
	fn get_next_move(&mut self, game: &TwoPlayerGame<B, M>) -> M;
}

pub struct ComputerPlayer<R: rand::Rng> {
	rng: R,
	nb_iter: usize,
	bias: f32,
}

impl<R: rand::Rng> ComputerPlayer<R> {
	pub fn new(rng: R, nb_iter: usize, bias: f32) -> ComputerPlayer<R> {
		ComputerPlayer {
			rng: rng,
			nb_iter: nb_iter,
			bias: bias,
		}
	}
}

impl<B: TwoPlayerBoard<M>, M: Move, R: rand::Rng> AppPlayer<B, M> for ComputerPlayer<R> {
	fn get_next_move(&mut self, game: &TwoPlayerGame<B, M>) -> M {
		mcts::search(game, &mut self.rng, self.nb_iter, self.bias)
	}
}

pub struct HumanPlayer;

impl<B: TwoPlayerBoard<M>, M: Move + Display + Input> AppPlayer<B, M> for HumanPlayer {
	fn get_next_move(&mut self, game: &TwoPlayerGame<B, M>) -> M {
		println!("Possible moves:");
		let moves = game.possible_moves();
		for m in &moves {
			println!("{}", m)
		}

		M::choose_stdin(&moves)
	}
}

pub trait Input {
	fn choose_stdin(moves: &Vec<Self>) -> Self;
}

pub struct App<B: TwoPlayerBoard<M>, M: Move> {
	game: TwoPlayerGame<B, M>,
	players: [Box<AppPlayer<B, M>>; 2],
}

impl<B: TwoPlayerBoard<M> + Display, M: Move + Display + Input> App<B, M> {
	pub fn new(board: B) -> App<B, M> {
		App {
			game: TwoPlayerGame::new(board),
			players: [make_player("White player"), make_player("Black player")],
		}
	}

	pub fn run(&mut self) {
        loop {
			print!("{}", self.game.board());

			if self.game.is_over() {
				break;
			}

			if self.game.current_player() == mcts::Player::White {
				println!("White turn");
			} else {
				println!("Black turn");
			};

			let moves = self.game.possible_moves();

			let m: M = if moves.len() == 1 {
				moves[0].clone()
			} else {
				self.players[self.game.current_player() as usize].get_next_move(&self.game)
			};

			println!("{:?} player played {}", self.game.current_player(), m);

			self.game.play(&m);
		}

		match self.game.winner() {
			Some(mcts::Player::White) => println!("White won"),
			Some(mcts::Player::Black) => println!("Blacks won"),
			None => println!("Draw"),
		}
	}

	pub fn run_quiet(&mut self) -> Option<mcts::Player> {
		while !self.game.is_over() {
			let moves = self.game.possible_moves();
			let m: M = if moves.len() == 1 {
				moves[0].clone()
			} else {
				self.players[self.game.current_player() as usize].get_next_move(&self.game)
			};
			self.game.play(&m);
		}

		return self.game.winner();
	}
}

fn make_player<B: TwoPlayerBoard<M>, M: Move + Display + Input>(msg: &str) -> Box<AppPlayer<B, M>> {
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
		0 => Box::new(HumanPlayer),
		_ => Box::new(ComputerPlayer::new(rand::thread_rng(), 10000, 1.0)),
	}
}
