use std::ops;

use crate::constants::*;
use crate::game::analysis::kingattackrayanalyzer::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::pieces::bishop::*;
use crate::game::pieces::king::*;
use crate::game::pieces::knight::*;
use crate::game::pieces::pawn::*;
use crate::game::pieces::piece::*;
use crate::game::pieces::queen::*;
use crate::game::pieces::rook::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct PositionAnalyzer {

}

impl PositionAnalyzer {
    pub fn calc_all_attacked_squares(position: &Position, player: &PlayerColour, enemy_king_pos: u64, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        match player {
            PlayerColour::WHITE => {
                Pawn::calc_attacked_squares(position, position.wp, &PlayerColour::WHITE, enemy_king_pos, king_attack_analyzer) |
                    Knight::calc_attacked_squares(position, position.wn, &PlayerColour::WHITE, enemy_king_pos, king_attack_analyzer) |
                    King::calc_attacked_squares(position, position.wk, &PlayerColour::WHITE, enemy_king_pos, king_attack_analyzer) |
                    Rook::calc_attacked_squares(position, position.wr, &PlayerColour::WHITE, enemy_king_pos, king_attack_analyzer) |
                    Bishop::calc_attacked_squares(position, position.wb, &PlayerColour::WHITE, enemy_king_pos, king_attack_analyzer) |
                    Queen::calc_attacked_squares(position, position.wq, &PlayerColour::WHITE, enemy_king_pos, king_attack_analyzer)
            }

            PlayerColour::BLACK => {
                Pawn::calc_attacked_squares(position, position.bp, &PlayerColour::BLACK, enemy_king_pos, king_attack_analyzer) |
                    Knight::calc_attacked_squares(position, position.bn, &PlayerColour::BLACK, enemy_king_pos, king_attack_analyzer) |
                    King::calc_attacked_squares(position, position.bk, &PlayerColour::BLACK, enemy_king_pos, king_attack_analyzer) |
                    Rook::calc_attacked_squares(position, position.br, &PlayerColour::BLACK, enemy_king_pos, king_attack_analyzer) |
                    Bishop::calc_attacked_squares(position, position.bb, &PlayerColour::BLACK, enemy_king_pos, king_attack_analyzer) |
                    Queen::calc_attacked_squares(position, position.bq, &PlayerColour::BLACK, enemy_king_pos, king_attack_analyzer)
            }
        }
    }

    #[inline(always)]
    pub fn update_position_from_king_ray_attack_analysis(position: &mut Position, king_attack_analyzer: &KingAttackRayAnalyzer) {
        position.king_in_check = king_attack_analyzer.num_checking_pieces > 0;
        position.king_in_double_check = king_attack_analyzer.num_checking_pieces > 1;
        position.en_passant_sq &= PositionHelper::bool_to_bitboard(!king_attack_analyzer.disable_en_passant);
        position.pin_ray_masks = king_attack_analyzer.pin_ray_masks;
        position.check_ray_mask = king_attack_analyzer.check_ray_mask;
    }

    pub fn calc_legal_moves(position: &mut Position, move_list: &mut GameMoveList) {
        let mut king_attack_analyzer = KingAttackRayAnalyzer::default();

        if position.white_to_move {
            let enemy_attacked_squares= PositionAnalyzer::calc_all_attacked_squares(position, &PlayerColour::BLACK, position.wk, &mut king_attack_analyzer);

            PositionAnalyzer::update_position_from_king_ray_attack_analysis(position, &king_attack_analyzer);

            let (_king_attacks, _king_movements) = King::calc_movements(position, position.wk, move_list, Some(enemy_attacked_squares), &mut king_attack_analyzer);
            if position.king_in_double_check { return; }

            let (_pawn_attacks, _pawn_movements) = Pawn::calc_movements(position, position.wp, move_list, None, &mut king_attack_analyzer);
            let (_knight_attacks, _knight_movements) = Knight::calc_movements(position, position.wn, move_list, None, &mut king_attack_analyzer);
            let (_rook_attacks, _rook_movements) = Rook::calc_movements(position, position.wr, move_list, None, &mut king_attack_analyzer);
            let (_bishop_attacks, _bishop_movements) = Bishop::calc_movements(position, position.wb, move_list, None, &mut king_attack_analyzer);
            let (_queen_attacks, _queen_movements) = Queen::calc_movements(position, position.wq, move_list, None, &mut king_attack_analyzer);

        } else {
            let enemy_attacked_squares = PositionAnalyzer::calc_all_attacked_squares(position, &PlayerColour::WHITE, position.bk, &mut king_attack_analyzer);

            PositionAnalyzer::update_position_from_king_ray_attack_analysis(position, &king_attack_analyzer);

            let (_king_attacks, _king_movements) = King::calc_movements(position, position.bk, move_list, Some(enemy_attacked_squares), &mut king_attack_analyzer);
            if position.king_in_double_check { return; }

            let (_pawn_attacks, _pawn_movements) = Pawn::calc_movements(position, position.bp, move_list, None, &mut king_attack_analyzer);
            let (_knight_attacks, _knight_movements) = Knight::calc_movements(position, position.bn, move_list, None, &mut king_attack_analyzer);
            let (_rook_attacks, _rook_movements) = Rook::calc_movements(position, position.br, move_list, None, &mut king_attack_analyzer);
            let (_bishop_attacks, _bishop_movements) = Bishop::calc_movements(position, position.bb, move_list, None, &mut king_attack_analyzer);
            let (_queen_attacks, _queen_movements) = Queen::calc_movements(position, position.bq, move_list, None, &mut king_attack_analyzer);
        }

        position.is_stalemate = move_list.list_len <= 0 && !position.king_in_check;
        position.is_checkmate = move_list.list_len <= 0 && position.king_in_check;
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use crate::test::legalmoveshelper::LegalMovesHelper;

    use super::*;

    #[test]
    fn test_calc_all_attacked_squares() {
        // 1. Starting position
        let (_enemy_attacked_squares, position, mut _move_list, mut king_attack_analyzer) = LegalMovesHelper::init_test_position_from_fen_str(None);
        let white_attacks = PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::WHITE, position.bk, &mut king_attack_analyzer);
        let black_attacks = PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::BLACK, position.wk, &mut king_attack_analyzer);

        assert_eq!(white_attacks, PositionHelper::bitboard_from_algebraic(vec!["b1", "c1", "d1", "e1", "f1", "g1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3"]));
        assert_eq!(black_attacks, PositionHelper::bitboard_from_algebraic(vec!["b8", "c8", "d8", "e8", "f8", "g8", "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6"]));

        LegalMovesHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            u64::MAX,
            0,
            false
        );

        // TODO: add something more exciting than just the starting position
    }

    #[test]
    fn test_double_check() {
        // 1. Starting position
        let mut position = Position::from_fen(Some("4k3/6N1/5b2/4R3/8/8/8/4K3 b KQkq b6 1 2")).unwrap();
        let mut move_list = GameMoveList::default();

        PositionAnalyzer::calc_legal_moves(&mut position, &mut move_list);

        assert_eq!(position.king_in_check, true);
        assert_eq!(position.king_in_double_check, true);

        for i in 0..move_list.list_len {
            assert_eq!(move_list.move_list[i].piece as u8, PieceType::KING as u8);
        }
    }

    #[test]
    fn test_pins() {
        // 1. One pinned piece
        let (enemy_attacked_squares, mut position, mut move_list, mut king_attack_analyzer) = LegalMovesHelper::init_test_position_from_fen_str(Some("4k3/4b3/8/8/4R3/8/8/4K3 b - - 1 2"));
        LegalMovesHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            LegalMovesHelper::generate_pin_ray_board(vec![("e7", vec!["e4", "e5", "e6", "e7"])]),
            u64::MAX,
            0,
            false
        );
        let (king_attacks, king_movements) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares), &mut king_attack_analyzer);
        let (bishop_attacks, bishop_movements) = Bishop::calc_movements(&position, position.bb, &mut move_list, None, &mut king_attack_analyzer);
        LegalMovesHelper::check_attack_and_movement_squares(
            (king_attacks | bishop_attacks, king_movements | bishop_movements),
            vec!["d8", "d7", "f8", "f7", "e7", "a3", "b4", "c5", "d6", "f6", "g5", "h4"],
            vec!["d8", "d7", "f8", "f7"]
        );

        LegalMovesHelper::switch_sides(&mut position, Some(&mut move_list), Some(&mut king_attack_analyzer));
        LegalMovesHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            u64::MAX,
            0,
            false
        );


        // 2. Two pinned pieces but one can be captured
        let (enemy_attacked_squares, position, mut move_list, mut king_attack_analyzer) = LegalMovesHelper::init_test_position_from_fen_str(Some("4k3/3pb3/2B5/8/4R3/8/8/4K3 b - - 1 2"));
        // PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::WHITE, position.bk, &mut king_attack_analyzer);
        LegalMovesHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            LegalMovesHelper::generate_pin_ray_board(vec![("d7", vec!["c6", "d7"]), ("e7", vec!["e4", "e5", "e6", "e7"])]),
            u64::MAX,
            0,
            false
        );

        let (pawn_attacks, pawn_movements) = Pawn::calc_movements(&position, position.bp, &mut move_list, None, &mut king_attack_analyzer);
        let (king_attacks, king_movements) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares), &mut king_attack_analyzer);
        let (bishop_attacks, bishop_movements) = Bishop::calc_movements(&position, position.bb, &mut move_list, None, &mut king_attack_analyzer);
        LegalMovesHelper::check_attack_and_movement_squares(
            (king_attacks | bishop_attacks | pawn_attacks, king_movements | bishop_movements | pawn_movements),
            vec!["d8", "d7", "f8", "f7", "e7", "c6", "e6", "a3", "b4", "c5", "d6", "f6", "g5", "h4"],
            vec!["d8", "f8", "f7", "c6"]
        );
    }

    #[test]
    fn test_checks() {
        // 1. Checking piece can be captured
        let (enemy_attacked_squares, position, mut move_list, mut king_attack_analyzer) = LegalMovesHelper::init_test_position_from_fen_str(Some("4k3/3p4/4Q3/8/4R3/8/8/4K3 b - - 1 2"));
        LegalMovesHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            PositionHelper::bitboard_from_algebraic(vec!["e6", "e7"]),
            1,
            false
        );

        let (pawn_attacks, pawn_movements) = Pawn::calc_movements(&position, position.bp, &mut move_list, None, &mut king_attack_analyzer);
        let (king_attacks, king_movements) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares), &mut king_attack_analyzer);
        LegalMovesHelper::check_attack_and_movement_squares(
            (king_attacks | pawn_attacks, king_movements | pawn_movements),
            vec!["d8", "d7", "f8", "f7", "e7", "e6"],
            vec!["d8", "f8", "e6"]
        );


        // 2. King cannot move backwards along path of checking rook (only left or right)
        let (enemy_attacked_squares, position, mut move_list, mut king_attack_analyzer) = LegalMovesHelper::init_test_position_from_fen_str(Some("8/4k3/8/8/4R3/8/8/4K3 b KQkq - 1 2"));
        LegalMovesHelper::check_king_attack_analysis(
            &king_attack_analyzer,
            [u64::MAX; 64],
            PositionHelper::bitboard_from_algebraic(vec!["e4", "e5", "e6"]),
            1,
            false
        );

        let (king_attacks, king_movements) = King::calc_movements(&position, position.bk, &mut move_list, Some(enemy_attacked_squares), &mut king_attack_analyzer);
        LegalMovesHelper::check_attack_and_movement_squares(
            (king_attacks, king_movements),
            vec!["d8", "d7", "d6", "e8", "e6", "f8", "f7", "f6"],
            vec!["d8", "d7", "d6", "f8", "f7", "f6"]
        );
    }

    #[test]
    fn test_calc_legal_moves_benchmark() {
        // currently about 8.5s after calculating and storing pawn moves only
        // About 20s after all pieces were added
        // let iterations = 10000000;
        let iterations = 100;

        let mut position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        let mut move_list = GameMoveList::default();
        let before = Instant::now();
        for _ in 0..iterations {
            move_list.clear();
            PositionAnalyzer::calc_legal_moves(&mut position, &mut move_list);
            // println!("{:?}", MoveGenerator::calc_legal_moves(&position).move_list);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
        // println!("{:?}", move_list);
    }

    // #[test]
    // // Result - trailing_zeros() is faster by about 33%
    // fn test_bit_scan_speed() {
    //     let before = Instant::now();
    //     for _ in 0..40000000 {
    //         assert_eq!(0xff0000u64.tzcnt() as u8, 16_u8);
    //     }
    //     println!("tzcnt - Elapsed time: {:.2?}", before.elapsed());
    //
    //     let before = Instant::now();
    //     for _ in 0..40000000 {
    //         assert_eq!(0xff0000u64.trailing_zeros() as u8, 16_u8);
    //     }
    //     println!("trailing_zeros - Elapsed time: {:.2?}", before.elapsed());
    // }
}