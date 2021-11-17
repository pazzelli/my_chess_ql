use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::movegenerator::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct Bishop {

}

impl Piece for Bishop {
    fn get_piece_type() -> PieceType { PieceType::BISHOP }

    fn calc_attacked_squares(position: &Position, mut piece_pos: u64, _player: &PlayerColour) -> u64 {
        let mut bishop_attacks = 0u64;

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            bishop_attacks |= Bishop::calc_file_or_diagonal_attacks(&position, sq_ind, DIAGONALS[sq_ind]);
            bishop_attacks |= Bishop::calc_file_or_diagonal_attacks(&position, sq_ind, ANTI_DIAGONALS[sq_ind]);

            piece_pos &= piece_pos - 1;
        }

        bishop_attacks
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;

    #[test]
    fn test_calc_bishop_movements() {
        let mut move_list = GameMoveList::default();

        // 1. Random position
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2pP1/3RKB1R w - - 1 2")).unwrap();
        let (attacked_squares, movement_squares) = Bishop::calc_movements(&position, position.wb, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["g2", "e2", "d3", "c4", "b5", "a6", "f6", "h6", "h4", "f4", "e3", "d2", "c1"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["e2", "d3", "c4", "b5", "a6", "f6", "h6", "h4", "f4", "e3", "d2", "c1"]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let (attacked_squares, movement_squares) = Bishop::calc_movements(&position, position.bb, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["f6", "f8", "h8", "h6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["h8", "h6"]));
    }
}