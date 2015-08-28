extern crate rand;

use std::fmt::{Formatter, Result, Debug};

use rand::Rng;

// TODO: 2 search methods (nb iter and time budget)
// TODO: maybe re-use nodes from one move to the other?

/// A `Game` represents a complete game state
#[derive(Clone)]
pub struct Game<G: TwoPlayerGame<M>, M: Move> {
	inner: G,
	current_player: Player,
	move_type: std::marker::PhantomData<M>,
}

impl<G: TwoPlayerGame<M>, M: Move> Game<G, M> {
	/// Create a complete game from the given partial game
	pub fn new(inner: G) -> Game<G, M> {
		Game {
			inner: inner,
			current_player: Player::White,
			move_type: std::marker::PhantomData,
		}
	}

	/// Actually plays the given move and update the game state accordingly
	///
	/// Note: that function doesn't check the validity of the given move
	pub fn play(&mut self, m: &M) {
		self.inner.play(self.current_player, m);
		self.current_player = self.current_player.opponent();
	}

	/// Convenience function that returns the possible moves in a new vector
	pub fn possible_moves(&self) -> Vec<M> {
		self.inner.possible_moves(self.current_player)
	}

	/// Return true is the game is over
	pub fn is_over(&self) -> bool {
		self.inner.possible_moves(self.current_player).is_empty()
	}

	/// Returns the winner of a terminated game
	///
	/// Note: that function must be called on a terminated game
	pub fn winner(&self) -> Option<Player> {
		self.inner.winner()
	}

	/// Returns the current player, i.e. the player that must play
	pub fn current_player(&self) -> Player {
		self.current_player
	}

	/// Returns a reference on the inner partial game state
	pub fn inner(&self) -> &G {
		&(self.inner)
	}
}

/// A `TwoPlayerGame` represent a partial game state for two-player games
///
/// This is the main trait to implement so that the game can be played by the AI
pub trait TwoPlayerGame<M: Move> : Clone {
	/// Appends every possible move for the given player in the given vector
	fn possible_moves_in(&self, p: Player, &mut Vec<M>);

	/// Actually plays the given move and update the game state accordingly
	fn play(&mut self, p: Player, m: &M);

	/// Returns the winner of a terminated game
	///
	/// Note: that function must be called on a terminated game
	fn winner(&self) -> Option<Player>;

	/// Convenience function that returns the possible moves in a new vector
	fn possible_moves(&self, p: Player) -> Vec<M> {
		let mut moves = Vec::new();
		self.possible_moves_in(p, &mut moves);
		return moves;
	}
}

/// A `Move` represents a move in a game.
pub trait Move: Debug+Clone+Default {}

/// A player in a two-player game
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Player {
	White,
	Black,
}

impl Player {
	/// Returns the opponent player
	pub fn opponent(&self) -> Player {
		// TODO: check if it needs explicit inlining
		match *self {
			Player::White => Player::Black,
			Player::Black => Player::White,
		}
	}
}

/// Automatically determines the best move to play from the given game
pub fn search<G, M, R>(game: &Game<G, M>, rng: &mut R, nb_iter: usize, bias: f32, debug: bool) -> M
	where G: TwoPlayerGame<M>, M: Move, R: rand::Rng {
	let mut root = Node::new(&M::default(), game, rng);

	for _ in 0..nb_iter {
		root.iter(&mut game.clone(), rng, bias);
	}

	// TODO: find the maximum value without sorting
	root.children.sort_by(|a, b| (b.score / b.nb_visits as f32).partial_cmp(&(a.score / a.nb_visits as f32)).unwrap_or(std::cmp::Ordering::Equal));

	if debug {
		println!("{:?}", root);
	}

	return root.children[0].last_move.clone();
}

/// Randomly plays a game until the end and return the winner
fn rollout<G, M, R>(game: &mut Game<G, M>, rng: &mut R) -> Option<Player>
	where G: TwoPlayerGame<M>, M: Move, R: rand::Rng {
	let mut moves = Vec::new();
	game.inner.possible_moves_in(game.current_player, &mut moves);

	while !moves.is_empty() {
		game.play(rng.choose(&moves[..]).unwrap());
		moves.clear();
		game.inner.possible_moves_in(game.current_player, &mut moves);
	}

	game.winner()
}

/// Internal structure for the MCTS algorithm
///
/// It is a partial tree of the game state space with statistics on the
/// outcomes of a game from a node.
#[derive(Clone)]
struct Node<M: Move> {
	last_move: M,
	children: Vec<Node<M>>,
	score: f32,
	nb_visits: usize,
	untried_moves: Vec<M>,
	last_player: Player,
}

impl<M: Move> Node<M> {
	fn new<G, R>(last_move: &M, game: &Game<G, M>, rng: &mut R) -> Node<M>
		where G: TwoPlayerGame<M>, R: rand::Rng {
		let mut untried_moves = game.possible_moves();
		// shuffle now so that we can pick a random value just by popping the last one
		rng.shuffle(&mut untried_moves[..]);

		Node {
			last_move: last_move.clone(),
			children: Vec::with_capacity(untried_moves.len()),
			score: 0.0,
			nb_visits: 0,
			untried_moves: untried_moves,
			last_player: game.current_player.opponent(),
		}
	}

	/// Find the child that maximize the UCB1 formula
	fn select_best_child(&mut self, bias: f32) -> &mut Node<M> {
		let mut best_value: f32 = std::f32::NEG_INFINITY;
		let mut best_child: Option<&mut Node<M>> = None;

		for c in &mut self.children {
			let value = c.score / c.nb_visits as f32 + bias * (2.0 * (self.nb_visits as f32).ln() / c.nb_visits as f32).sqrt();
			if value > best_value {
				best_value = value;
				best_child = Some(c);
			}
		}

		best_child.unwrap()
	}

	/// Add a child to the current node with an previously unexplored move
	fn expand<G, R>(&mut self, game: &mut Game<G, M>, rng: &mut R) -> Option<Player>
		where G: TwoPlayerGame<M>, R: rand::Rng {
		let m: M = self.untried_moves.pop().unwrap();
		game.play(&m);

		let mut child = Node::new(&m, game, rng);
		let winner = rollout(game, rng);
		child.update(winner);
		self.children.push(child);

		return winner;
	}

	/// Update the node according to the outcome of the game
	fn update(&mut self, winner: Option<Player>) {
		self.score += match winner {
			Some(p) => if p == self.last_player { 1.0 } else { 0.0 },
			None => 0.5,
		};
		self.nb_visits += 1;
	}

	/// Perform one iteration of the UCT algorithm (MCTS+UCB1)
	fn iter<G, R>(&mut self, game: &mut Game<G, M>, rng: &mut R, bias: f32) -> Option<Player>
		where G: TwoPlayerGame<M>, R: rand::Rng {
		let w = if self.untried_moves.is_empty() {
			if self.children.is_empty() {
				// Terminal node
				game.winner()
			} else {
				// Select
				let child = self.select_best_child(bias);
				game.play(&child.last_move);
				child.iter(game, rng, bias)
			}
		} else {
			// Expand
			self.expand(game, rng)
		};

		// Update
		self.update(w);
		return w;
	}
}

impl<M: Move> Debug for Node<M> {
	fn fmt(&self, f: &mut Formatter) -> Result {
		fn print<M: Move>(node: &Node<M>, f: &mut Formatter, depth: usize) -> Result {
			if depth <= 1 {
				for _ in 0..depth {
					try!(write!(f, "  "));
				}

				if depth == 0 {
					try!(write!(f, "[root]\n"));
				} else {
					try!(write!(f, "[M: {:?}, S/V: {}/{} = {}]\n", node.last_move, node.score, node.nb_visits, node.score / node.nb_visits as f32));
				}

				for c in node.children.iter() {
					try!(print(c, f, depth + 1));
				}
			}
			return Ok(());
		}

		print(self, f, 0)
	}
}
