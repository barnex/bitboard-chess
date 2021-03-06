use std::{convert::TryFrom, fmt::Write};

use super::internal::*;

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Square {
	Empty = 0,

	WPawn = 1,
	WRook = 2,
	WKnight = 3,
	WBisshop = 4,
	WQueen = 5,
	WKing = 6,

	BPawn = 7,
	BRook = 8,
	BKnight = 9,
	BBisshop = 10,
	BQueen = 11,
	BKing = 12,
}

use Square::*;

impl Square {
	pub const ALL_PIECES: [Square; 12] = [
		WPawn, WRook, WKnight, WBisshop, WQueen, WKing, BPawn, BRook, BKnight, BBisshop, BQueen, BKing,
	];
	pub const ALL_SQUARES: [Square; 13] = [
		Empty, WPawn, WRook, WKnight, WBisshop, WQueen, WKing, BPawn, BRook, BKnight, BBisshop, BQueen, BKing,
	];
	const ASCII: [char; 14] = ['.', 'P', 'R', 'N', 'B', 'Q', 'K', 'p', 'r', 'n', 'b', 'q', 'k', '?'];
	const UNICODE: [char; 14] = [' ', '♙', '♖', '♘', '♗', '♕', '♔', '♟', '♜', '♞', '♝', '♛', '♚', '?'];

	/// Piece representation following https://en.wikipedia.org/wiki/Forsyth%E2%80%93Edwards_Notation.
	/// `None` is represented as '.', `OffBoard` as `?`.
	pub fn to_char(self) -> char {
		self.into()
	}

	pub fn is_empty(self) -> bool {
		self == Empty
	}

	pub fn color(self) -> Option<Color> {
		use Color::*;
		match self {
			WPawn | WRook | WKnight | WBisshop | WQueen | WKing => Some(White),
			BPawn | BRook | BKnight | BBisshop | BQueen | BKing => Some(Black),
			Empty => None,
		}
	}

	pub fn sign(self) -> i32 {
		match self {
			WPawn | WRook | WKnight | WBisshop | WQueen | WKing => 1,
			BPawn | BRook | BKnight | BBisshop | BQueen | BKing => -1,
			Empty => 0,
		}
	}

	pub const fn index(self) -> usize {
		self as usize
	}

	pub fn is_color(self, color: Color) -> bool {
		self.color() == Some(color)
	}

	pub fn is_king(self) -> bool {
		match self {
			WKing | BKing => true,
			_ => false,
		}
	}

	pub fn unicode(self) -> char {
		Self::UNICODE[self.index()]
	}

	/// https://en.wikipedia.org/wiki/Chess_piece_relative_value.
	pub fn value(self) -> i32 {
		Self::VALUE[self.index()]
	}

	const VALUE: [i32; 13] = [0, 1, 5, 3, 3, 9, 0, -1, -5, -3, -3, -9, 0];
}

impl fmt::Display for Square {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.write_char(self.to_char())
	}
}

impl Into<char> for Square {
	fn into(self) -> char {
		Self::ASCII[self.index()]
	}
}

impl TryFrom<char> for Square {
	type Error = anyhow::Error;

	fn try_from(value: char) -> Result<Self> {
		use Square::*;
		Ok(match value {
			'.' => Empty,
			'P' => WPawn,
			'R' => WRook,
			'N' => WKnight,
			'B' => WBisshop,
			'Q' => WQueen,
			'K' => WKing,
			'p' => BPawn,
			'r' => BRook,
			'n' => BKnight,
			'b' => BBisshop,
			'q' => BQueen,
			'k' => BKing,
			invalid => return Err(format_err!("invalid piece: {}", invalid)),
		})
	}
}

impl Default for Square {
	fn default() -> Self {
		Square::Empty
	}
}
