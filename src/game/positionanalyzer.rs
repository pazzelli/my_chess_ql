use std::ops;
use crate::constants::*;
use crate::game::pieces::piece::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::pieces::pawn::*;
use crate::game::pieces::knight::*;
use crate::game::pieces::king::*;
use crate::game::pieces::rook::*;
use crate::game::pieces::bishop::*;
use crate::game::pieces::queen::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct KingAttackRayAnalysis(pub u64, pub u64, pub u8, pub bool);

impl Default for KingAttackRayAnalysis {
    fn default() -> Self {
        KingAttackRayAnalysis(0u64, 0u64, 0, false)
    }
}

impl ops::AddAssign<KingAttackRayAnalysis> for KingAttackRayAnalysis {
    fn add_assign(&mut self, _rhs: KingAttackRayAnalysis) {
        self.0 |= _rhs.0;
        self.1 |= _rhs.1;
        self.2 += _rhs.2;
        self.3 |= _rhs.3;
    }
}

pub struct PositionAnalyzer {

}

impl PositionAnalyzer {
    pub fn calc_all_attacked_squares(position: &Position, player: &PlayerColour, enemy_king_pos: u64) -> (u64, KingAttackRayAnalysis) {
        let mut attacked_squares = 0u64;
        let mut king_attack_analysis = KingAttackRayAnalysis::default();

        // TODO: clean-up this function perhaps using a helper function?

        match player {
            PlayerColour::WHITE => {
                let (temp_attacked_squares, temp_king_attacks) = Pawn::calc_attacked_squares(position, position.wp, &PlayerColour::WHITE, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, temp_king_attacks) = Knight::calc_attacked_squares(position, position.wn, &PlayerColour::WHITE, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, _) = King::calc_attacked_squares(position, position.wk, &PlayerColour::WHITE, enemy_king_pos);
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, temp_king_attacks) = Rook::calc_attacked_squares(position, position.wr, &PlayerColour::WHITE, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, temp_king_attacks) = Bishop::calc_attacked_squares(position, position.wb, &PlayerColour::WHITE, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, temp_king_attacks) = Queen::calc_attacked_squares(position, position.wq, &PlayerColour::WHITE, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;
            }

            PlayerColour::BLACK => {
                let (temp_attacked_squares, temp_king_attacks) = Pawn::calc_attacked_squares(position, position.bp, &PlayerColour::BLACK, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, temp_king_attacks) = Knight::calc_attacked_squares(position, position.bn, &PlayerColour::BLACK, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, _) = King::calc_attacked_squares(position, position.bk, &PlayerColour::BLACK, enemy_king_pos);
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, temp_king_attacks) = Rook::calc_attacked_squares(position, position.br, &PlayerColour::BLACK, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, temp_king_attacks) = Bishop::calc_attacked_squares(position, position.bb, &PlayerColour::BLACK, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;

                let (temp_attacked_squares, temp_king_attacks) = Queen::calc_attacked_squares(position, position.bq, &PlayerColour::BLACK, enemy_king_pos);
                king_attack_analysis += temp_king_attacks;
                attacked_squares |= temp_attacked_squares;
            }
        }
        (attacked_squares, king_attack_analysis)
    }

    pub fn calc_legal_moves(position: &mut Position, move_list: &mut GameMoveList) {
        // let mut move_list = GameMoveList::default();

        if position.white_to_move {
            let (enemy_attacked_squares, king_attack_analysis) = PositionAnalyzer::calc_all_attacked_squares(position, &PlayerColour::BLACK, position.wk);
            let KingAttackRayAnalysis(pinning_pieces, king_checks, num_checking_pieces, disable_en_passant) = king_attack_analysis;

            position.king_in_check = num_checking_pieces > 0;
            position.king_in_double_check = num_checking_pieces > 1;

            let (_king_attacks, _king_movements) = King::calc_movements(position, position.wk, move_list, Some(enemy_attacked_squares));
            if position.king_in_double_check { return; }

            let (_pawn_attacks, _pawn_movements) = Pawn::calc_movements(position, position.wp, move_list, None);
            let (_knight_attacks, _knight_movements) = Knight::calc_movements(position, position.wn, move_list, None);
            let (_rook_attacks, _rook_movements) = Rook::calc_movements(position, position.wr, move_list, None);
            let (_bishop_attacks, _bishop_movements) = Bishop::calc_movements(position, position.wb, move_list, None);
            let (_queen_attacks, _queen_movements) = Queen::calc_movements(position, position.wq, move_list, None);

        } else {
            let (enemy_attacked_squares, king_attack_analysis) = PositionAnalyzer::calc_all_attacked_squares(position, &PlayerColour::WHITE, position.bk);
            let KingAttackRayAnalysis(pinning_pieces, king_checks, num_checking_pieces, disable_en_passant) = king_attack_analysis;

            position.king_in_check = num_checking_pieces > 0;
            position.king_in_double_check = num_checking_pieces > 1;

            let (_king_attacks, _king_movements) = King::calc_movements(position, position.bk, move_list, Some(enemy_attacked_squares));
            if position.king_in_double_check { return; }

            let (_pawn_attacks, _pawn_movements) = Pawn::calc_movements(position, position.bp, move_list, None);
            let (_knight_attacks, _knight_movements) = Knight::calc_movements(position, position.bn, move_list, None);
            let (_rook_attacks, _rook_movements) = Rook::calc_movements(position, position.br, move_list, None);
            let (_bishop_attacks, _bishop_movements) = Bishop::calc_movements(position, position.bb, move_list, None);
            let (_queen_attacks, _queen_movements) = Queen::calc_movements(position, position.bq, move_list, None);
        }
        // move_list
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;
    
    #[test]
    fn test_calc_all_attacked_squares() {
        // 1. Starting position
        let position = Position::from_fen(None).unwrap();
        let (white_attacks, _num_pieces_giving_check) = PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::WHITE, position.bk);
        let (black_attacks, king_attack_analysis) = PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::BLACK, position.wk);
        let KingAttackRayAnalysis(pinning_pieces, king_checks, num_checking_pieces, disable_en_passant) = king_attack_analysis;

        assert_eq!(white_attacks, PositionHelper::bitboard_from_algebraic(vec!["b1", "c1", "d1", "e1", "f1", "g1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3"]));
        assert_eq!(black_attacks, PositionHelper::bitboard_from_algebraic(vec!["b8", "c8", "d8", "e8", "f8", "g8", "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6"]));
        assert_eq!(pinning_pieces, 0);
        assert_eq!(king_checks, 0);
        assert_eq!(num_checking_pieces, 0);
        assert_eq!(disable_en_passant, false);

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
        // let mut move_list = GameMoveList::default();

        // TODO: add checks to ensure pinned piece movement is restricted

        // 1. One pinned piece
        let position = Position::from_fen(Some("4k3/4b3/8/8/4R3/8/8/4K3 b KQkq - 1 2")).unwrap();
        let (_white_attacks, white_king_attack_analysis) = PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::WHITE, position.bk);
        let (_black_attacks, black_king_attack_analysis) = PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::BLACK, position.wk);

        let KingAttackRayAnalysis(pinning_pieces, king_checks, num_checking_pieces, disable_en_passant) = white_king_attack_analysis;
        assert_eq!(pinning_pieces, PositionHelper::bitboard_from_algebraic(vec!["e7"]));
        assert_eq!(king_checks, 0);
        assert_eq!(num_checking_pieces, 0);
        assert_eq!(disable_en_passant, false);

        let KingAttackRayAnalysis(pinning_pieces, king_checks, num_checking_pieces, disable_en_passant) = black_king_attack_analysis;
        assert_eq!(pinning_pieces, 0);
        assert_eq!(king_checks, 0);
        assert_eq!(num_checking_pieces, 0);
        assert_eq!(disable_en_passant, false);


        // 2. Two pinned pieces but one can be captured
        let position = Position::from_fen(Some("4k3/3pb3/2B5/8/4R3/8/8/4K3 b KQkq - 1 2")).unwrap();
        let (_white_attacks, white_king_attack_analysis) = PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::WHITE, position.bk);

        let KingAttackRayAnalysis(pinning_pieces, king_checks, num_checking_pieces, disable_en_passant) = white_king_attack_analysis;
        assert_eq!(pinning_pieces, PositionHelper::bitboard_from_algebraic(vec!["d7", "e7"]));
        assert_eq!(king_checks, 0);
        assert_eq!(num_checking_pieces, 0);
        assert_eq!(disable_en_passant, false);
    }

    #[test]
    fn test_checks() {
        // TODO: add checks to ensure enemy movement is restricted to deal with the check

        // 1. Checking piece can be captured
        let mut position = Position::from_fen(Some("4k3/3p4/4Q3/8/4R3/8/8/4K3 b KQkq - 1 2")).unwrap();
        let mut move_list = GameMoveList::default();
        let (_white_attacks, king_attack_analysis) = PositionAnalyzer::calc_all_attacked_squares(&position, &PlayerColour::WHITE, position.bk);
        PositionAnalyzer::calc_legal_moves(&mut position, &mut move_list);

        let KingAttackRayAnalysis(pinning_pieces, king_checks, num_checking_pieces, disable_en_passant) = king_attack_analysis;
        assert_eq!(pinning_pieces, 0);
        assert_eq!(king_checks, PositionHelper::bitboard_from_algebraic(vec!["e6", "e7"]));
        assert_eq!(num_checking_pieces, 1);
        assert_eq!(disable_en_passant, false);
    }

    #[test]
    fn test_calc_legal_moves_benchmark() {
        // currently about 8.5s after calculating and storing pawn moves only
        // About 20s after all pieces were added
        let iterations = 10000000;
        // let iterations = 100;

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