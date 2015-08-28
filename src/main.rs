extern crate mcts;

#[macro_use]
mod grid;
mod checkers;
mod app;

fn main() {
	app::App::new(checkers::Board::new()).run();
}
