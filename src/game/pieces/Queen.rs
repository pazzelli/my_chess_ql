use std::ops::DerefMut;
use crate::constants::*;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::pieces::bishop::Bishop;
use crate::game::pieces::piece::*;
use crate::game::pieces::rook::Rook;
use crate::game::position::Position;
use crate::game::PIECE_ATTACK_SQUARES;

pub struct Queen {

}

impl Piece for Queen {
    fn get_piece_type() -> PieceType { PieceType::QUEEN }

    // TODO: come up with a more efficient implementation for queen movements (using SIMD?)
    fn calc_attacked_squares(position: &Position, mut piece_pos: u64, _player: &PlayerColour, enemy_king_pos: u64, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        let mut queen_attacks = 0u64;

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            let mut cur_piece_attacks = Rook::calc_rank_attacks(&position, sq_ind, RANKS[sq_ind], enemy_king_pos);
            cur_piece_attacks |= Rook::calc_file_or_diagonal_attacks(&position, sq_ind, FILES[sq_ind], enemy_king_pos);
            cur_piece_attacks |= Bishop::calc_file_or_diagonal_attacks(&position, sq_ind, DIAGONALS[sq_ind], enemy_king_pos);
            cur_piece_attacks |= Bishop::calc_file_or_diagonal_attacks(&position, sq_ind, ANTI_DIAGONALS[sq_ind], enemy_king_pos);

            queen_attacks |= cur_piece_attacks;

            PIECE_ATTACK_SQUARES.with(|attack_squares| {
                attack_squares.borrow_mut().deref_mut()[sq_ind] = cur_piece_attacks;
            });

            if RANKS[sq_ind] & enemy_king_pos > 0 || FILES[sq_ind] & enemy_king_pos > 0 || DIAGONALS[sq_ind] & enemy_king_pos > 0 || ANTI_DIAGONALS[sq_ind] & enemy_king_pos > 0 {
                let king_sq = enemy_king_pos.trailing_zeros() as usize;
                king_attack_analyzer.analyze_king_attack_ray(position, ATTACK_RAYS[(sq_ind << 6) + king_sq], RANKS[sq_ind] & enemy_king_pos > 0, enemy_king_pos);
            }

            piece_pos &= piece_pos - 1;
        }

        queen_attacks & position.check_ray_mask
    }
}

#[cfg(test)]
mod tests {
    use crate::test::legalmoveshelper::LegalMovesTestHelper;

    use super::*;

    #[test]
    fn test_calc_queen_movements() {
        // 1. Starting position
        let (_, mut position, mut move_list, mut king_attack_analyzer, _) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/3RKB1R w - - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Queen::calc_movements(&position, position.wq, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["c1", "a2", "b2", "b3", "c3", "d3", "e3", "f3", "g3", "a4", "b4", "a5", "c5", "a6", "a7"],
            "a3a4 a3a5 a3a6 a3a7 a3b2 a3b3 a3b4 a3c1 a3c3 a3c5 a3d3 a3e3 a3f3 a3g3"
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Queen::calc_movements(&position, position.bq, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["a8", "f8", "e7", "b8", "c8", "e8", "a5", "b6", "c7", "d7", "d6", "d5"],
            "d8a5 d8b6 d8b8 d8c7 d8c8 d8d5 d8d6 d8d7 d8e8"
        );
    }
}