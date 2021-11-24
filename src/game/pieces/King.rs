use crate::constants::*;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::moves::gamemove::*;
use crate::game::moves::gamemovelist::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct King {

}

impl Piece for King {
    fn get_piece_type() -> PieceType { PieceType::KING }

    fn calc_attacked_squares(_position: &Position, piece_pos: u64, _player: &PlayerColour, _enemy_king_pos: u64, _king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        let sq_ind: usize = piece_pos.trailing_zeros() as usize;
        KING_ATTACKS[sq_ind]
    }

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by the king
    //  movementSquares = squares where the king can either move or capture a piece
    #[inline(always)]
    fn calc_movements(position: &Position, piece_pos: u64, move_list: &mut GameMoveList, enemy_attacked_squares: u64, _king_attack_analyzer: &mut KingAttackRayAnalyzer) -> (u64, u64) {
        // Cannot use the base implementation since the king also cannot move into check

        // Squares not controlled by the enemy side (needed because the king cannot move into check)
        let sq_ind: usize = piece_pos.trailing_zeros() as usize;

        // Need to flip the check ray mask for the king only, since the king cannot move into the check ray, but other pieces
        // can move in to block it
        // The XOR flips the check ray mask to disallow any squares along the check ray, but then the
        // OR with the friendly occupancy brings back the single square along that ray containing the checking piece
        let check_ray_mask_for_king = (PositionHelper::bool_to_bitboard(position.king_in_check) ^ position.check_ray_mask) | position.enemy_occupancy;
        let king_valid_squares = KING_ATTACKS[sq_ind] & !enemy_attacked_squares & check_ray_mask_for_king;
        let king_captures = king_valid_squares & position.enemy_occupancy;
        let king_non_captures = king_valid_squares & position.non_occupancy;

        // For castling, both squares between the king and castling square must not be occupied
        // Also, the king must still be on e1 or e8 and not in check, and the target castling
        // squares must not be occupied or attacked by any enemy piece
        // Note: position.castling_rights must be updated externally when a move is made and here is assumed
        // to be a bitboard containing valid, remaining castling squares for both sides
        let castle_base = king_non_captures | (
            position.castling_rights
                & PositionHelper::bool_to_bitboard(sq_ind == 4 || sq_ind == 60)
                & PositionHelper::bool_to_bitboard(!position.king_in_check)
                & !enemy_attacked_squares
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
    use crate::test::legalmoveshelper::LegalMovesTestHelper;

    use super::*;

    #[test]
    fn test_calc_king_movements() {
        // 1. Starting position
        let (enemy_attacked_squares, mut position, mut move_list, mut king_attack_analyzer, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.wk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["d1", "d2", "e2", "f2", "f1"],
            vec![]
        );

        let enemy_attacked_squares = LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), Some(&mut king_attack_analyzer));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.bk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["d8", "d7", "e7", "f7", "f8"],
            vec![]
        );


        // 2. Position with friendly pieces in the way, one pawn to capture, one empty square that is enemy-controlled
        let (enemy_attacked_squares, mut position, mut move_list, mut king_attack_analyzer, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5nP/P1P2pP1/3RKB1R w KQkq - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.wk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["d1", "d2", "e2", "f2", "f1"],
            vec!["d2", "f2"]
        );
        LegalMovesTestHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            PositionHelper::bitboard_from_algebraic(vec!["f2"]),
            1,
            false
        );


        let enemy_attacked_squares = LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), Some(&mut king_attack_analyzer));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.bk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["f8", "f7", "g7", "h7", "h8"],
            vec!["h8"]
        );
        LegalMovesTestHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            u64::MAX,
            0,
            false
        );



        // 3. Position with rook and enemy king where king cannot escape to a hidden square along the sliding attack ray
        let (enemy_attacked_squares, position, mut move_list, mut king_attack_analyzer, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("8/4k3/8/8/4R3/8/8/4K3 b KQkq - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.bk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["d8", "d7", "d6", "f8", "f7", "f6", "e6", "e8"],
            vec!["d8", "d7", "d6", "f8", "f7", "f6"]
        );
        LegalMovesTestHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            PositionHelper::bitboard_from_algebraic(vec!["e4", "e5", "e6"]),
            1,
            false
        );

        // println!("{:?}", move_list);
    }

    #[test]
    fn test_castling() {
        // 1. Position with both sides having castling rights, black king in check, black bishop controlling
        // one of white's kingside castling squares
        let (enemy_attacked_squares, mut position, mut move_list, mut king_attack_analyzer, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r3k2r/8/8/8/8/3bQ3/8/R3K2R w KQkq - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.wk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["d1", "d2", "e2", "f2", "f1"],
            vec!["d1", "d2", "f2", "c1"]
        );
        LegalMovesTestHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            u64::MAX,
            0,
            false
        );

        let enemy_attacked_squares = LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), Some(&mut king_attack_analyzer));
        position.king_in_check = true;
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.bk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["f8", "f7", "e7", "d7", "d8"],
            vec!["d8", "d7", "f8", "f7"]
        );
        LegalMovesTestHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            PositionHelper::bitboard_from_algebraic(vec!["e3", "e4", "e5", "e6", "e7"]),
            1,
            false
        );


        // 2. More complicated position
        let (enemy_attacked_squares, mut position, mut move_list, mut king_attack_analyzer, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r3k1nr/8/8/8/3bb3/8/8/R3K2R w KQkq - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.wk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["d1", "d2", "e2", "f2", "f1"],
            vec!["d1", "d2", "e2", "f1", "c1"]
        );

        let enemy_attacked_squares = LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), Some(&mut king_attack_analyzer));
        // position.king_in_check = true;
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.bk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            vec!["f8", "f7", "e7", "d7", "d8"],
            vec!["f8", "f7", "e7", "d7", "d8", "c8"]
        );

        // println!("{:?}", move_list);
    }
}