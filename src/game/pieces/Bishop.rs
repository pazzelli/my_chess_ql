use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::movegenerator::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct Bishop {

}

impl Bishop {
    #[inline(always)]
    fn add_bishop_movement(move_list: &mut GameMoveList, source_square: u8, mut target_squares: u64, is_capture: bool) {
        while target_squares > 0 {
            // trailing_zeros() gives square index from 0..63
            move_list.add_move(PieceType::BISHOP, source_square, target_squares.trailing_zeros() as u8, is_capture, PieceType::NONE);
            target_squares &= target_squares - 1;
        }
    }
}

impl Piece for Bishop {
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

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by the bishop
    //  movementSquares = squares where the bishop can either move or capture a piece
    #[inline(always)]
    fn calc_movements(position: &Position, mut piece_pos: u64, move_list: &mut GameMoveList, _enemy_attacked_squares: Option<u64>) -> (u64, u64) {
        let attacked_squares = Bishop::calc_attacked_squares(position, piece_pos, if position.white_to_move {&PlayerColour::WHITE} else {&PlayerColour::BLACK});

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;

            let capture_squares = attacked_squares & position.enemy_occupancy;
            let non_capture_squares = attacked_squares & position.non_occupancy;
            Bishop::add_bishop_movement(move_list, sq_ind as u8, capture_squares, true);
            Bishop::add_bishop_movement(move_list, sq_ind as u8, non_capture_squares, false);

            piece_pos &= piece_pos - 1;
        }

        (attacked_squares, attacked_squares & !position.friendly_occupancy)
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

        // 1. Starting position
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


        // // 2. Position with friendly pieces in the way, one pawn to capture, one empty square that is enemy-controlled
        // let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2pP1/3RKB1R w KQkq - 1 2")).unwrap();
        // move_list.clear();
        // let (attacked_squares, movement_squares) = Rook::calc_movements(&position, position.wr, &mut move_list, None);
        // assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a1", "b1", "c1", "d2", "d3", "d4", "d5", "e1", "f1", "g1", "h2", "h3"]));
        // assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a1", "b1", "c1", "d2", "d3", "d4", "g1", "h2", "h3"]));
        //
        // position.white_to_move = false;
        // position.update_occupancy();
        // move_list.clear();
        // let (attacked_squares, movement_squares) = Rook::calc_movements(&position, position.br, &mut move_list, None);
        // assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a7", "b8", "c8", "d8", "e8", "f7", "g8"]));
        // assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["b8", "c8", "e8"]));
    }
}