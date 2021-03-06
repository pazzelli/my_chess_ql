use std::ops::DerefMut;
use crate::constants::*;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::PIECE_ATTACK_SQUARES;

pub struct Rook {

}

impl Piece for Rook {
    fn get_piece_type() -> PieceType { PieceType::ROOK }

    fn calc_attacked_squares(position: &Position, mut piece_pos: u64, _player: &PlayerColour, enemy_king_pos: u64, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        let mut rook_attacks = 0u64;

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            let mut cur_piece_attacks = Rook::calc_rank_attacks(&position, sq_ind, RANKS[sq_ind], enemy_king_pos);
            cur_piece_attacks |= Rook::calc_file_or_diagonal_attacks(&position, sq_ind, FILES[sq_ind], enemy_king_pos);
            rook_attacks |= cur_piece_attacks;

            PIECE_ATTACK_SQUARES.with(|attack_squares| {
                attack_squares.borrow_mut().deref_mut()[sq_ind] = cur_piece_attacks;
            });

            if RANKS[sq_ind] & enemy_king_pos > 0 || FILES[sq_ind] & enemy_king_pos > 0 {
                let king_sq = enemy_king_pos.trailing_zeros() as usize;
                king_attack_analyzer.analyze_king_attack_ray(position, ATTACK_RAYS[(sq_ind << 6) + king_sq], RANKS[sq_ind] & enemy_king_pos > 0, enemy_king_pos);
            }

            piece_pos &= piece_pos - 1;
        }

        rook_attacks & position.check_ray_mask
    }
}

#[cfg(test)]
mod tests {
    use crate::test::legalmoveshelper::LegalMovesTestHelper;

    use super::*;

    #[test]
    fn test_calc_rook_movements() {
        // 1. Starting position
        let (_, mut position, mut move_list, mut king_attack_analyzer, _) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("8/8/2KB4/8/k7/8/Pb1r4/R2N4 w - - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Rook::calc_movements(&position, position.wr, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["b1", "c1", "d1", "a2"],
            "a1b1 a1c1"
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Rook::calc_movements(&position, position.br, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["d1", "c2", "b2", "e2", "f2", "h2", "g2", "d3", "d4", "d5", "d6"],
            "d2c2 d2d1 d2d3 d2d4 d2d5 d2d6 d2e2 d2f2 d2g2 d2h2"
        );


        // 2. Position with friendly pieces in the way, one pawn to capture, one empty square that is enemy-controlled
        let (_, mut position, mut move_list, _, _) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/3RKB1R w KQkq - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Rook::calc_movements(&position, position.wr, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["a1", "b1", "c1", "d2", "d3", "d4", "d5", "e1", "f1", "g1", "h2", "h3"],
            "d1a1 d1b1 d1c1 d1d2 d1d3 d1d4 h1g1 h1h2 h1h3"
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Rook::calc_movements(&position, position.br, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["a7", "b8", "c8", "d8", "e8", "f7", "g8"],
            "a8b8 a8c8 f8e8"
        );
    }
}