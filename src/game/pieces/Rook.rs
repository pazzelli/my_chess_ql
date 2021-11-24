use crate::constants::*;
use crate::game::analysis::positionanalyzer::*;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::moves::gamemove::*;
use crate::game::moves::gamemovelist::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct Rook {

}

impl Piece for Rook {
    fn get_piece_type() -> PieceType { PieceType::ROOK }

    fn calc_attacked_squares(position: &Position, mut piece_pos: u64, _player: &PlayerColour, enemy_king_pos: u64, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        let mut rook_attacks = 0u64;

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;

            rook_attacks |= Rook::calc_rank_attacks(&position, sq_ind, RANKS[sq_ind], enemy_king_pos);
            rook_attacks |= Rook::calc_file_or_diagonal_attacks(&position, sq_ind, FILES[sq_ind], enemy_king_pos);

            if RANKS[sq_ind] & enemy_king_pos > 0 {
                let king_sq = enemy_king_pos.trailing_zeros() as usize;
                king_attack_analyzer.analyze_king_attack_ray(position, ATTACK_RAYS[(sq_ind << 6) + king_sq], true, enemy_king_pos);
            }

            if FILES[sq_ind] & enemy_king_pos > 0 {
                let king_sq = enemy_king_pos.trailing_zeros() as usize;
                king_attack_analyzer.analyze_king_attack_ray(position, ATTACK_RAYS[(sq_ind << 6) + king_sq], false, enemy_king_pos);
            }

            piece_pos &= piece_pos - 1;
        }

        rook_attacks & position.check_ray_mask
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use crate::test::legalmoveshelper::LegalMovesTestHelper;

    use super::*;

    #[test]
    fn test_calc_rook_movements() {
        // 1. Starting position
        let (_, mut position, mut move_list, mut king_attack_analyzer, _) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("8/8/2KB4/8/k7/8/Pb1r4/R2N4 w - - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Rook::calc_movements(&position, position.wr, &mut move_list, 0, &mut king_attack_analyzer),
            vec!["b1", "c1", "d1", "a2"],
            vec!["b1", "c1"]
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Rook::calc_movements(&position, position.br, &mut move_list, 0, &mut king_attack_analyzer),
            vec!["d1", "c2", "b2", "e2", "f2", "h2", "g2", "d3", "d4", "d5", "d6"],
            vec!["d1", "c2", "e2", "f2", "h2", "g2", "d3", "d4", "d5", "d6"]
        );


        // 2. Position with friendly pieces in the way, one pawn to capture, one empty square that is enemy-controlled
        let (_, mut position, mut move_list, _, _) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/3RKB1R w KQkq - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Rook::calc_movements(&position, position.wr, &mut move_list, 0, &mut king_attack_analyzer),
            vec!["a1", "b1", "c1", "d2", "d3", "d4", "d5", "e1", "f1", "g1", "h2", "h3"],
            vec!["a1", "b1", "c1", "d2", "d3", "d4", "g1", "h2", "h3"]
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Rook::calc_movements(&position, position.br, &mut move_list, 0, &mut king_attack_analyzer),
            vec!["a7", "b8", "c8", "d8", "e8", "f7", "g8"],
            vec!["b8", "c8", "e8"]
        );
    }
}