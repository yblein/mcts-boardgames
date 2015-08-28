extern crate rand;
extern crate mcts;

use std;
use std::io::Write;
use std::fmt::Display;

use mcts::{Game, TwoPlayerGame, Move};

/// `Input` represents data that can be read from the standard input
pub trait Input {
	/// Asks the user to choose a move among the given ones
	fn choose_stdin(moves: &Vec<Self>) -> Self;
}

/// `App` is the full application
///
/// The application asks for the player types (Human or Computer) and
/// interactively runs a game.
pub struct App<G: TwoPlayerGame<M>, M: Move> {
	game: Game<G, M>,
	players: [Box<AppPlayer<G, M>>; 2],
}

impl<G: TwoPlayerGame<M> + Display, M: Move + Display + Input> App<G, M> {
	/// Create a new application for the given game
	pub fn new(game: G) -> App<G, M> {
		App {
			game: Game::new(game),
			players: [make_player("White player"), make_player("Black player")],
		}
	}

	/// Interactively runs the game to the end.
	pub fn run(&mut self) {
        loop {
			print!("{}", self.game.inner());

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

	/// Runs the game to the end without performing any IO
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

fn make_player<G: TwoPlayerGame<M>, M: Move + Display + Input>(msg: &str) -> Box<AppPlayer<G, M>> {
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
		_ => Box::new(ComputerPlayer::new(rand::thread_rng(), 10000, 0.71)),
	}
}

trait AppPlayer<G: TwoPlayerGame<M>, M: Move> {
	fn get_next_move(&mut self, game: &Game<G, M>) -> M;
}

struct ComputerPlayer<R: rand::Rng> {
	rng: R,
	nb_iter: usize,
	bias: f32,
}

impl<R: rand::Rng> ComputerPlayer<R> {
	fn new(rng: R, nb_iter: usize, bias: f32) -> ComputerPlayer<R> {
		ComputerPlayer {
			rng: rng,
			nb_iter: nb_iter,
			bias: bias,
		}
	}
}

impl<G: TwoPlayerGame<M>, M: Move, R: rand::Rng> AppPlayer<G, M> for ComputerPlayer<R> {
	fn get_next_move(&mut self, game: &Game<G, M>) -> M {
		mcts::search(game, &mut self.rng, self.nb_iter, self.bias, false)
	}
}

struct HumanPlayer;

impl<G: TwoPlayerGame<M>, M: Move + Display + Input> AppPlayer<G, M> for HumanPlayer {
	fn get_next_move(&mut self, game: &Game<G, M>) -> M {
		println!("Possible moves:");
		let moves = game.possible_moves();
		for m in &moves {
			println!("{}", m)
		}

		M::choose_stdin(&moves)
	}
}
