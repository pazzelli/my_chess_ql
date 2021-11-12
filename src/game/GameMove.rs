use std::fmt::*;
use crate::uci::engine::constants::*;
use crate::uci::engine::position::Position;
use crate::uci::engine::position_helper::PositionHelper;

// #[derive(Clone, Copy, Debug)]
#[derive(Clone, Copy)]
pub struct GameMove {
    pub piece: Piece,
    pub source_square: u8,
    pub target_square: u8,
    pub promotion_piece: Piece,
    pub is_capture: bool
}

impl Default for GameMove {
    fn default() -> Self {
        GameMove {
            piece: Piece::NONE,
            source_square: 0,
            target_square: 0,
            promotion_piece: Piece::NONE,
            is_capture: false
        }
    }
}

impl Debug for GameMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("GameMove")
            .field("piece", &self.piece)
            .field("source_square", &PositionHelper::algebraic_from_index(self.source_square))
            .field("target_square", &PositionHelper::algebraic_from_index(self.target_square))
            .field("promotion_piece", &self.promotion_piece)
            .field("is_capture", &self.is_capture)
            .finish()
    }
}
