extern crate rand;

use rand::Rng;

// TODO: 2 search methods (nb iter and time budget)
// TODO: maybe re-use nodes from one move to the other?

#[derive(Clone)]
pub struct TwoPlayerGame<B: TwoPlayerBoard<M>, M: Move> {
	board: B,
	current_player: Player,
	move_type: std::marker::PhantomData<M>,
}

impl<B: TwoPlayerBoard<M>, M: Move> TwoPlayerGame<B, M> {
	pub fn new(board: B) -> TwoPlayerGame<B, M> {
		TwoPlayerGame {
			board: board,
			current_player: Player::White,
			move_type: std::marker::PhantomData,
		}
	}

	pub fn play(&mut self, m: &M) {
		self.board.play(self.current_player, m);
		self.current_player = self.current_player.opponent();
	}

	pub fn possible_moves(&self) -> Vec<M> {
		self.board.possible_moves(self.current_player)
	}

	pub fn is_over(&self) -> bool {
		self.board.possible_moves(self.current_player).is_empty()
	}

	pub fn winner(&self) -> Option<Player> {
		self.board.winner()
	}

	pub fn current_player(&self) -> Player {
		self.current_player
	}

	pub fn board(&self) -> &B {
		&(self.board)
	}
}

pub trait TwoPlayerBoard<M: Move> : Clone {
	fn possible_moves_in(&self, p: Player, &mut Vec<M>);

	fn play(&mut self, p: Player, m: &M);

	fn winner(&self) -> Option<Player>;

	fn possible_moves(&self, p: Player) -> Vec<M> {
		let mut moves = Vec::new();
		self.possible_moves_in(p, &mut moves);
		return moves;
	}
}

pub trait Move: std::fmt::Debug+std::fmt::Display+Clone+Eq+Default {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Player {
	White,
	Black,
}

impl Player {
	pub fn opponent(&self) -> Player {
		// TODO: check if it needs inlining
		match *self {
			Player::White => Player::Black,
			Player::Black => Player::White,
		}
	}
}

pub fn search<B, M, R>(game: &TwoPlayerGame<B, M>, rng: &mut R, nb_iter: usize, bias: f32) -> M
	where B: TwoPlayerBoard<M>, M: Move, R: rand::Rng {
	//for _ in 0..10 {
	let mut root = Node::new(&M::default(), game, rng);

	for _ in 0..nb_iter {
		root.uct(&mut game.clone(), rng, bias);
	}

	/*
	root.children.sort_by(|a, b| b.nb_visits.cmp(&a.nb_visits));
	let best = &root.children[0];
	println!("win chances: {}", best.score / best.nb_visits as f32);
	root.print(0);
	}
	panic!("");
	*/

	/*
	let (v1, v2) = root.children.split_at(1);
	let mut best_child = &v1[0];

	for c in v2 {
		if (c.score / c.nb_visits as f32) > (best_child.score / best_child.nb_visits as f32) {
			best_child = c;
		}
	}

	return best_child.last_move.clone();
	*/

	root.children.sort_by(|a, b| (b.score / b.nb_visits as f32).partial_cmp(&(a.score / a.nb_visits as f32)).unwrap_or(std::cmp::Ordering::Equal));
	println!("{:?}", root);
	return root.children[0].last_move.clone();
}

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
	fn new<B, R>(last_move: &M, game: &TwoPlayerGame<B, M>, rng: &mut R) -> Node<M>
		where B: TwoPlayerBoard<M>, R: rand::Rng {
		let mut untried_moves = game.board.possible_moves(game.current_player);
		// shuffle now so that we can pick a random value just by popping the last one
		rng.shuffle(&mut untried_moves[..]);

		Node {
			last_move: last_move.clone(),
			//parent: parent,
			children: Vec::with_capacity(untried_moves.len()),
			score: 0.0,
			nb_visits: 0,
			untried_moves: untried_moves,
			last_player: game.current_player.opponent(),
		}
	}

	fn select_child_ucb1(&mut self, bias: f32) -> &mut Node<M> {
		/* Use the UCB1 formula to select a child node. Often a constant UCTK is applied so we have
		   lambda c: c.score/c.nb_visits + UCTK * sqrt(2*log(self.nb_visits)/c.nb_visits to vary the amount of
		   exploration versus exploitation. */

		let mut best_value: f32 = std::f32::NEG_INFINITY;
		let mut best_child: Option<&mut Node<M>> = None;
		//let bias: f32 = 2f32.sqrt();
		//let bias: f32 = 1f32;

		for c in &mut self.children {
			let value = c.score / c.nb_visits as f32 + bias * (2.0 * (self.nb_visits as f32).ln() / c.nb_visits as f32).sqrt();
			if value > best_value {
				best_value = value;
				best_child = Some(c);
			}
		}

		best_child.unwrap()
	}

	fn add_child<B, R>(&mut self, last_move: &M, game: &TwoPlayerGame<B, M>, rng: &mut R) -> &mut Node<M>
		where B: TwoPlayerBoard<M>, R: rand::Rng {
		let child = Node::new(last_move, game, rng);
		self.children.push(child);
		return self.children.last_mut().unwrap();
	}

	fn update(&mut self, winner: Option<Player>) {
		self.score += match winner {
			Some(p) => if p == self.last_player { 1.0 } else { 0.0 },
			None => 0.5,
		};
		self.nb_visits += 1;
	}

	fn uct<B, R>(&mut self, game: &mut TwoPlayerGame<B, M>, rng: &mut R, bias: f32) -> Option<Player>
		where B: TwoPlayerBoard<M>, R: rand::Rng {
		let w = if self.untried_moves.is_empty() {
			if self.children.is_empty() {
				// Terminal node
				game.board.winner()
			} else {
				// Select
				let child = self.select_child_ucb1(bias);
				game.play(&child.last_move);
				child.uct(game, rng, bias)
			}
		} else {
			// Expand
			let m: M = self.untried_moves.pop().unwrap();
			game.play(&m);
			let node = self.add_child(&m, game, rng);
			let w = rollout(game, rng);
			node.update(w);
			w
		};

		self.update(w);
		return w;
	}
}

impl<M: Move> std::fmt::Debug for Node<M> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		fn print<M: Move>(node: &Node<M>, f: &mut std::fmt::Formatter, depth: usize) -> std::fmt::Result {
			if depth <= 1 {
				for _ in 0..depth {
					try!(write!(f, "  "));
				}

				if depth == 0 {
					try!(write!(f, "[root]\n"));
				} else {
					try!(write!(f, "[M: {}, S/V: {}/{}]\n", node.last_move, node.score, node.nb_visits));
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

fn rollout<B, M, R>(game: &mut TwoPlayerGame<B, M>, rng: &mut R) -> Option<Player>
	where B: TwoPlayerBoard<M>, M: Move, R: rand::Rng {
	let mut moves = Vec::new();
	game.board.possible_moves_in(game.current_player, &mut moves);

	while !moves.is_empty() {
		game.play(rng.choose(&moves[..]).unwrap());
		moves.clear();
		game.board.possible_moves_in(game.current_player, &mut moves);
	}

	game.board.winner()
}
