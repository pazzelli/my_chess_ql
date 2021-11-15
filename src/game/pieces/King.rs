use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::movegenerator::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct King {

}

impl King {
    #[inline(always)]
    fn add_king_movement(move_list: &mut GameMoveList, source_square: u8, mut target_squares: u64, is_capture: bool) {
        while target_squares > 0 {
            // trailing_zeros() gives square index from 0..63
            move_list.add_move(PieceType::KING, source_square, target_squares.trailing_zeros() as u8, is_capture, PieceType::NONE);
            target_squares &= target_squares - 1;
        }
    }
}

impl Piece for King {
    fn calc_attacked_squares(_position: &Position, piece_pos: u64, _player: PlayerColour) -> u64 {
        let sq_ind: usize = piece_pos.trailing_zeros() as usize;
        KING_ATTACKS[sq_ind]
    }

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by the king
    //  movementSquares = squares where the king can either move or capture a piece
    #[inline(always)]
    fn calc_movements(position: &Position, piece_pos: u64, move_list: &mut GameMoveList, enemy_attacked_squares: Option<u64>) -> (u64, u64) {
        // Squares not controlled by the enemy side (needed because the king cannot move into check)
        let enemy_non_attacks = !(enemy_attacked_squares.unwrap());
        let sq_ind: usize = piece_pos.trailing_zeros() as usize;

        let king_valid_squares = KING_ATTACKS[sq_ind] & enemy_non_attacks;
        let king_captures = king_valid_squares & position.enemy_occupancy;
        let king_non_captures = king_valid_squares & position.non_occupancy;

        King::add_king_movement(move_list, sq_ind as u8, king_captures, true);
        King::add_king_movement(move_list, sq_ind as u8, king_non_captures, false);

        (KING_ATTACKS[sq_ind], king_captures | king_non_captures)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;

    #[test]
    fn test_calc_king_movements() {
        let mut move_list = GameMoveList::default();

        // 1. Starting position
        let mut position = Position::from_fen(None).unwrap();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, PlayerColour::BLACK);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.wk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "d2", "e2", "f2", "f1"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec![]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, PlayerColour::WHITE);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d8", "d7", "e7", "f7", "f8"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec![]));


        // 2. Position with friendly pieces in the way, one pawn to capture, one empty square that is enemy-controlled
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5nP/P1P2pP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.clear();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, PlayerColour::BLACK);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.wk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "d2", "e2", "f2", "f1"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["d2", "f2"]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, PlayerColour::WHITE);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["f8", "f7", "g7", "h7", "h8"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["h8"]));
    }
}