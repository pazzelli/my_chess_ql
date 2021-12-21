// Needed to disambiguate source piece for each possible move
use core::cell::RefCell;
thread_local!(static PIECE_ATTACK_SQUARES: RefCell<[u64; 64]> = RefCell::new([0u64; 64]));

pub mod pieces;
pub mod analysis;
pub mod moves;

pub mod position;
pub mod positionhelper;
