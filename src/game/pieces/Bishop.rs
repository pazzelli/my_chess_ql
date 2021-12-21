use std::ops::DerefMut;
use crate::constants::*;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::pieces::piece::*;
use crate::game::position::*;
use crate::game::PIECE_ATTACK_SQUARES;

pub struct Bishop {

}

impl Piece for Bishop {
    fn get_piece_type() -> PieceType { PieceType::BISHOP }

    fn calc_attacked_squares(position: &Position, mut piece_pos: u64, _player: &PlayerColour, enemy_king_pos: u64, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        let mut bishop_attacks = 0u64;

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            let mut cur_piece_attacks = Bishop::calc_file_or_diagonal_attacks(&position, sq_ind, DIAGONALS[sq_ind], enemy_king_pos);
            cur_piece_attacks |= Bishop::calc_file_or_diagonal_attacks(&position, sq_ind, ANTI_DIAGONALS[sq_ind], enemy_king_pos);
            bishop_attacks |= cur_piece_attacks;

            PIECE_ATTACK_SQUARES.with(|attack_squares| {
                attack_squares.borrow_mut().deref_mut()[sq_ind] = cur_piece_attacks;
            });

            if DIAGONALS[sq_ind] & enemy_king_pos > 0 || ANTI_DIAGONALS[sq_ind] & enemy_king_pos > 0 {
                let king_sq = enemy_king_pos.trailing_zeros() as usize;
                king_attack_analyzer.analyze_king_attack_ray(position, ATTACK_RAYS[(sq_ind << 6) + king_sq], false, enemy_king_pos);
            }

            piece_pos &= piece_pos - 1;
        }

        bishop_attacks & position.check_ray_mask
    }
}

#[cfg(test)]
mod tests {
    use crate::test::legalmoveshelper::LegalMovesTestHelper;

    use super::*;

    #[test]
    fn test_calc_bishop_movements() {
        let (_, mut position, mut move_list, mut king_attack_analyzer, _) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/3RKB1R w - - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Bishop::calc_movements(&position, position.wb, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["g2", "e2", "d3", "c4", "b5", "a6", "f6", "h6", "h4", "f4", "e3", "d2", "c1"],
            "f1a6 f1b5 f1c4 f1d3 f1e2 g5c1 g5d2 g5e3 g5f4 g5f6 g5h4 g5h6"
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Bishop::calc_movements(&position, position.bb, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["f6", "f8", "h8", "h6"],
            "g7h6 g7h8"
        );
    }
}