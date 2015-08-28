extern crate mcts;

mod utils;
mod checkers;
mod app;

fn main() {
	app::App::new(checkers::Board::new()).run();
}
