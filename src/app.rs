extern crate rand;
extern crate mcts;

use mcts::TwoPlayerGame;
use mcts::TwoPlayerBoard;
use mcts::Move;

use std::fmt::Display;

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

pub struct App<B: TwoPlayerBoard<M> + Display, M: Move> {
	game: TwoPlayerGame<B, M>,
	players: [Box<AppPlayer<B, M>>; 2],
}

impl<B: TwoPlayerBoard<M> + Display, M: Move> App<B, M> {
	pub fn new(board: B, p1: Box<AppPlayer<B, M>>, p2: Box<AppPlayer<B, M>>) -> App<B, M> {
		App {
			game: TwoPlayerGame::new(board),
			players: [p1, p2],
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
        loop {
			if self.game.is_over() {
				return self.game.winner();
			}

			let moves = self.game.possible_moves();

			let m: M = if moves.len() == 1 {
				moves[0].clone()
			} else {
				self.players[self.game.current_player() as usize].get_next_move(&self.game)
			};

			self.game.play(&m);
		}
	}
}
