use crate::constants::*;
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

impl GameMove {
    fn get_piece_type_letter(piece_type: PieceType) -> String {
        String::from(match piece_type {
            PieceType::PAWN => "P",
            PieceType::KNIGHT => "N",
            PieceType::BISHOP => "B",
            PieceType::ROOK => "R",
            PieceType::QUEEN => "Q",
            PieceType::KING => "K",
            _ => ""
        })
    }

    pub fn get_extended_san_move_string(&self) -> String {
        if self.piece == PieceType::KING {
            let square_diff = self.source_square as i16 - self.target_square as i16;
            if square_diff == 2 { return String::from("O-O-O"); }
            if square_diff == -2 { return String::from("O-O"); }
        }

        String::from(
            format! (
                "{}{}{}{}{}",
                GameMove::get_piece_type_letter(self.piece),
                PositionHelper::algebraic_from_index(self.source_square),
                if self.is_capture { "x" } else { "" },
                PositionHelper::algebraic_from_index(self.target_square),
                if self.promotion_piece != PieceType::NONE {format!("={}", GameMove::get_piece_type_letter(self.promotion_piece))} else { String::from("") }
            )
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_move_extended_san_notation() {
        let mut g = GameMove {
            piece: PieceType::KNIGHT,
            source_square: 6,
            target_square: 21,
            promotion_piece: PieceType::NONE,
            is_capture: false
        };
        assert_eq!(g.get_extended_san_move_string(), "Ng1f3");

        g = GameMove {
            piece: PieceType::BISHOP,
            source_square: 0,
            target_square: 63,
            promotion_piece: PieceType::NONE,
            is_capture: true
        };
        assert_eq!(g.get_extended_san_move_string(), "Ba1xh8");

        g = GameMove {
            piece: PieceType::PAWN,
            source_square: 53,
            target_square: 62,
            promotion_piece: PieceType::KNIGHT,
            is_capture: true
        };
        assert_eq!(g.get_extended_san_move_string(), "Pf7xg8=N");
    }
}