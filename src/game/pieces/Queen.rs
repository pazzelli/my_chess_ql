use crate::constants::*;
use crate::game::analysis::positionanalyzer::*;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::moves::gamemove::*;
use crate::game::moves::gamemovelist::*;
use crate::game::pieces::bishop::Bishop;
use crate::game::pieces::piece::*;
use crate::game::pieces::rook::Rook;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct Queen {

}

impl Piece for Queen {
    fn get_piece_type() -> PieceType { PieceType::QUEEN }

    // TODO: come up with a more efficient implementation for queen movements (using SIMD?)
    fn calc_attacked_squares(position: &Position, piece_pos: u64, player: &PlayerColour, enemy_king_pos: u64, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        Bishop::calc_attacked_squares(position, piece_pos, player, enemy_king_pos, king_attack_analyzer) |
            Rook::calc_attacked_squares(position, piece_pos, player, enemy_king_pos, king_attack_analyzer)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use crate::test::legalmoveshelper::LegalMovesTestHelper;

    use super::*;

    #[test]
    fn test_calc_queen_movements() {
        // 1. Starting position
        let (_, mut position, mut move_list, mut king_attack_analyzer, _) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/3RKB1R w - - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Queen::calc_movements(&position, position.wq, &mut move_list, 0, &mut king_attack_analyzer),
            vec!["c1", "a2", "b2", "b3", "c3", "d3", "e3", "f3", "g3", "a4", "b4", "a5", "c5", "a6", "a7"],
            vec!["c1", "b2", "b3", "c3", "d3", "e3", "f3", "g3", "a4", "b4", "a5", "c5", "a6", "a7"]
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Queen::calc_movements(&position, position.bq, &mut move_list, 0, &mut king_attack_analyzer),
            vec!["a8", "f8", "e7", "b8", "c8", "e8", "a5", "b6", "c7", "d7", "d6", "d5"],
            vec!["b8", "c8", "e8", "a5", "b6", "c7", "d7", "d6", "d5"]
        );
    }
}