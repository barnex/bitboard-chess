use super::internal::*;
use rayon::prelude::*;

/// Parallel alpha-beta
pub struct ParAlphaBeta<F: Fn(&Board, Color) -> i32 + Sync> {
	depth: u32,
	leaf_value: F,
}
impl<F: Fn(&Board, Color) -> i32 + Sync> ParAlphaBeta<F> {
	pub fn new(depth: u32, leaf_value: F) -> Self {
		Self { leaf_value, depth }
	}
}

impl<F: Fn(&Board, Color) -> i32 + Sync> Engine for ParAlphaBeta<F> {
	fn eval_moves(&self, board: &Board, player: Color) -> SmVec<(Move, i32)> {
		board
			.iter_moves(player)
			.map(|mv| (mv, board.with_move(mv)))
			.filter(|(_, board)| !board.is_check(player))
			.collect::<Vec<_>>()
			.par_iter()
			.map(|(mv, board)| (*mv, -alphabeta(&board, player.opposite(), &self.leaf_value, self.depth)))
			.collect::<Vec<_>>()
			.into_iter()
			.collect::<SmVec<_>>()
	}
}

/// How good is board for player?
/// Good scores are always positive, bad scores always negative,
/// regardless of player color.
pub fn alphabeta<F>(board: &Board, player: Color, leaf_eval: &F, depth: u32) -> i32
where
	F: Fn(&Board, Color) -> i32,
{
	alphabeta_(board, player, leaf_eval, -INF, INF, depth).1
}

pub fn alphabeta_<F>(board: &Board, player: Color, leaf_eval: &F, alpha: i32, beta: i32, depth: u32) -> (Option<Move>, i32)
where
	F: Fn(&Board, Color) -> i32,
{
	// must stop iteration so that we would not trade a king for a king :-)
	if !board.has_king(player) {
		return (None, -INF + (depth as i32));
	}
	// Not possible if we make no illegal moves:
	// if !board.has_king(player.opposite()) {
	// 	unreachable!();
	// }

	if depth == 0 {
		return (None, leaf_eval(board, player));
	}

	let mut mv_boards = board //
		.collect_moves(player)
		.iter()
		.map(|&mv| (mv, board.with_move(mv)))
		.collect::<Vec<_>>();

	// sorting moves most promising first
	// results in massively better alpha-beta pruning
	// but is only worth the cost at least two levels above leaf.
	if depth > 1 {
		let mut mv_board_value = mv_boards
			.into_iter()
			.map(|(mv, board)| {
				let value = leaf_eval(&board, player);
				(mv, board, value)
			})
			.collect::<SmVec<_>>();
		mv_board_value.sort_by_key(|(_, _, v)| *v);
		mv_boards = mv_board_value.into_iter().map(|(mv, board, _)| (mv, board)).collect();
	}

	let mut best_value = -INF - (depth as i32);
	let mut best_move = None;
	let mut alpha = alpha;
	for (mv, board) in mv_boards {
		// TODO: filter out bad moves at board level.
		if board.is_check(player) {
			continue;
		}

		let (_, value) = alphabeta_(&board, player.opposite(), leaf_eval, -beta, -alpha, depth - 1);
		let value = -value;
		if value >= best_value {
			best_value = value;
			best_move = Some(mv);
		}

		alpha = i32::max(alpha, value);
		if alpha >= beta {
			break;
		}
	}
	(best_move, best_value)
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn test_alphabeta_mate() {
		let board = board(
			r"
		. . . . R . . k
		. . . . R . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . K
		",
		);

		debug_assert_eq!(alphabeta(&board, White, &material, 0), 10);
		debug_assert_eq!(alphabeta(&board, Black, &material, 0), -10);

		debug_assert_eq!(alphabeta(&board, White, &material, 1), INF);
		//debug_assert_eq!(alphabeta(&board, Black, &material, 2), -INF+2);
	}

	#[test]
	fn test_alpahbeta_1() {
		let board = board(
			r"
		. . . . . . . .
		. . . . . p . .
		. . . . p . . .
		. . . . . . . .
		. . . . . . . .
		. . . . Q . . .
		. . . . . . . .
		k . . . . . . K
		",
		);

		debug_assert_eq!(alphabeta(&board, White, &material, 0), 7);
		debug_assert_eq!(alphabeta(&board, Black, &material, 0), -7);

		debug_assert_eq!(alphabeta(&board, White, &material, 1), 8); // white greedily takes a pawn
		debug_assert_eq!(alphabeta(&board, Black, &material, 1), -7);

		debug_assert_eq!(alphabeta(&board, White, &material, 2), 7); // white sees that the pawn is protected
	}

	#[test]
	fn test_alphabeta_2() {
		let board = board(
			r"
		Q . . p p p . k
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . K
		",
		);

		debug_assert_eq!(alphabeta(&board, White, &material, 0), 6); // no moves
		debug_assert_eq!(alphabeta(&board, Black, &material, 0), -6); // no moves

		debug_assert_eq!(alphabeta(&board, White, &material, 1), 7); // white: Qd8xp
		debug_assert_eq!(alphabeta(&board, Black, &material, 1), -6); // black: no moves

		debug_assert_eq!(alphabeta(&board, White, &material, 2), 7); // white: Qd8xp,    black: no moves
		debug_assert_eq!(alphabeta(&board, Black, &material, 2), -7); // black: no moves, white: Qd8xp

		debug_assert_eq!(alphabeta(&board, White, &material, 3), 8); // white: Qd8xp,   black: no moves, white: Qe8xp
		debug_assert_eq!(alphabeta(&board, Black, &material, 3), -7); // black: no moves, white: Qd8xp,  black: no moves

		debug_assert_eq!(alphabeta(&board, White, &material, 4), 8); // white: Qd8xp,   black: no moves, white: Qe8xp,    black: no moves
		debug_assert_eq!(alphabeta(&board, Black, &material, 4), -8); // black: no moves, white: Qd8xp,  black: no moves, white: Qe8xp
	}

	#[test]
	fn test_alphabeta_3() {
		let board = board(
			r"
		. . . . . . . k
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		. . . . . . . .
		q . . P P P . K
		",
		);

		debug_assert_eq!(alphabeta(&board, Black, &material, 0), 6);
		debug_assert_eq!(alphabeta(&board, White, &material, 0), -6);

		debug_assert_eq!(alphabeta(&board, Black, &material, 1), 7);
		debug_assert_eq!(alphabeta(&board, White, &material, 1), -6);

		debug_assert_eq!(alphabeta(&board, Black, &material, 2), 7);
		debug_assert_eq!(alphabeta(&board, White, &material, 2), -7);

		debug_assert_eq!(alphabeta(&board, Black, &material, 3), 8);
		debug_assert_eq!(alphabeta(&board, White, &material, 3), -7);

		debug_assert_eq!(alphabeta(&board, Black, &material, 4), 8);
		debug_assert_eq!(alphabeta(&board, White, &material, 4), -8);
	}

	fn board(board: &str) -> Board {
		Board::from_str(board).unwrap()
	}
}
