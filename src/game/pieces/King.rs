use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::movegenerator::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct King {

}

impl Piece for King {
    fn get_piece_type() -> PieceType { PieceType::KING }

    fn calc_attacked_squares(_position: &Position, piece_pos: u64, _player: &PlayerColour) -> u64 {
        let sq_ind: usize = piece_pos.trailing_zeros() as usize;
        KING_ATTACKS[sq_ind]
    }

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by the king
    //  movementSquares = squares where the king can either move or capture a piece
    #[inline(always)]
    fn calc_movements(position: &Position, piece_pos: u64, move_list: &mut GameMoveList, enemy_attacked_squares: Option<u64>) -> (u64, u64) {
        // Cannot use the base implementation since the king also cannot move into check

        // Squares not controlled by the enemy side (needed because the king cannot move into check)
        let enemy_attack_squares = enemy_attacked_squares.unwrap();
        let sq_ind: usize = piece_pos.trailing_zeros() as usize;

        // Note: position.castling_rights must be updated externally when a move is made and here is assumed
        // to be a bitboard containing valid, remaining castling squares for both sides
        let king_valid_squares = KING_ATTACKS[sq_ind] & !enemy_attack_squares;
        let king_captures = king_valid_squares & position.enemy_occupancy;
        let king_non_captures = king_valid_squares & position.non_occupancy;

        // For castling, both squares between the king and castling square must not be occupied
        // Also, the king must still be on e1 or e8 and not in check, and the target castling
        // squares must not be occupied or attacked by any enemy piece
        let castle_base = king_non_captures | (
            position.castling_rights
                & PositionHelper::bool_to_bitboard(sq_ind == 4 || sq_ind == 60)
                & PositionHelper::bool_to_bitboard(!position.king_in_check)
                & !enemy_attack_squares
                & position.non_occupancy
        );

        // let castle_base = (king_non_captures | position.castling_rights) & PositionHelper::bool_to_bitboard(!position.king_in_check);
        let short_castle = castle_base & (castle_base << 1);
        let long_castle = castle_base & (castle_base >> 1);
        let castling_squares = (short_castle | long_castle) & position.castling_rights;


        King::add_piece_movement(move_list, sq_ind as u8, king_captures, 0, true);
        King::add_piece_movement(move_list, sq_ind as u8, king_non_captures | castling_squares, castling_squares, false);

        (KING_ATTACKS[sq_ind], king_captures | king_non_captures | castling_squares)
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
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, &PlayerColour::BLACK);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.wk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "d2", "e2", "f2", "f1"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec![]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, &PlayerColour::WHITE);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d8", "d7", "e7", "f7", "f8"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec![]));


        // 2. Position with friendly pieces in the way, one pawn to capture, one empty square that is enemy-controlled
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5nP/P1P2pP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.clear();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, &PlayerColour::BLACK);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.wk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "d2", "e2", "f2", "f1"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["d2", "f2"]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, &PlayerColour::WHITE);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["f8", "f7", "g7", "h7", "h8"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["h8"]));
    }

    #[test]
    fn test_castling() {
        let mut move_list = GameMoveList::default();

        // 1. Position with both sides having castling rights, black king in check, black bishop controlling
        // one of white's kingside castling squares
        let mut position = Position::from_fen(Some("r3k2r/8/8/8/8/3bQ3/8/R3K2R w KQkq - 1 2")).unwrap();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, &PlayerColour::BLACK);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.wk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "d2", "e2", "f2", "f1"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "d2", "f2", "c1"]));

        position.white_to_move = false;
        position.king_in_check = true;
        position.update_occupancy();
        move_list.clear();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, &PlayerColour::WHITE);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["f8", "f7", "e7", "d7", "d8"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["d8", "d7", "f8", "f7"]));



        // 2. More complicated position
        let mut position = Position::from_fen(Some("r3k1nr/8/8/8/3bb3/8/8/R3K2R w KQkq - 1 2")).unwrap();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, &PlayerColour::BLACK);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.wk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "d2", "e2", "f2", "f1"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "d2", "e2", "f1", "c1"]));

        position.white_to_move = false;
        // position.king_in_check = true;
        position.update_occupancy();
        move_list.clear();
        let enemy_attacked_squares = MoveGenerator::calc_all_attacked_squares(&position, &PlayerColour::WHITE);
        let (attacked_squares, movement_squares) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["f8", "f7", "e7", "d7", "d8"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["f8", "f7", "e7", "d7", "d8", "c8"]));

        println!("{:?}", move_list);
    }
}