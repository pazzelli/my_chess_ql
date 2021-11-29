use crate::constants::*;
use crate::game::position::*;
use crate::game::positionhelper::*;
use std::fmt::*;

// #[derive(Clone, Copy, Debug)]
#[derive(Clone, Copy)]
pub struct GameMove {
    pub piece: PieceType,
    pub source_square: u8,
    pub target_square: u8,
    pub promotion_piece: PieceType,
    pub is_capture: bool
}

impl Default for GameMove {
    fn default() -> Self {
        GameMove {
            piece: PieceType::NONE,
            source_square: 0,
            target_square: 0,
            promotion_piece: PieceType::NONE,
            is_capture: false
        }
    }
}

impl Debug for GameMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut result = vec![PositionHelper::algebraic_from_index(self.source_square), PositionHelper::algebraic_from_index(self.target_square)];
        if self.promotion_piece != PieceType::NONE {
            result.push(String::from(match self.promotion_piece {
                PieceType::QUEEN => "q",
                PieceType::KNIGHT => "n",
                PieceType::ROOK => "r",
                PieceType::BISHOP => "b",
                _ => ""
            }));
        }
        f.write_str(result.join("").as_str())

        // f.debug_struct("GameMove")
        //     .field("piece", &self.piece)
        //     .field("source_square", &PositionHelper::algebraic_from_index(self.source_square))
        //     .field("target_square", &PositionHelper::algebraic_from_index(self.target_square))
        //     .field("promotion_piece", &self.promotion_piece)
        //     .field("is_capture", &self.is_capture)
        //     .finish()
    }
}
