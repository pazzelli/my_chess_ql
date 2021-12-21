use std::ops::DerefMut;
use crate::constants::*;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;
use crate::game::PIECE_ATTACK_SQUARES;

pub struct Knight {

}

impl Piece for Knight {
    fn get_piece_type() -> PieceType { PieceType::KNIGHT }

    fn calc_attacked_squares(position: &Position, mut piece_pos: u64, _player: &PlayerColour, enemy_king_pos: u64, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        let mut knight_attacks: u64 = 0;
        let mut king_check_board = 0u64;

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            knight_attacks |= KNIGHT_ATTACKS[sq_ind];

            PIECE_ATTACK_SQUARES.with(|attack_squares| {
                attack_squares.borrow_mut().deref_mut()[sq_ind] = KNIGHT_ATTACKS[sq_ind];
            });

            king_check_board |= SINGLE_BITBOARDS[sq_ind] & PositionHelper::bool_to_bitboard(KNIGHT_ATTACKS[sq_ind] & enemy_king_pos > 0);
            piece_pos &= piece_pos - 1;
        }

        king_attack_analyzer.merge_check_ray(king_check_board);

        knight_attacks & position.check_ray_mask
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;
    use crate::game::moves::gamemovelist::GameMoveList;
    use crate::test::legalmoveshelper::LegalMovesTestHelper;

    use super::*;

    #[test]
    fn test_calc_knight_movements() {
        // 1. Starting position
        let (_, mut position, mut move_list, mut king_attack_analyzer, _) = LegalMovesTestHelper::init_test_position_from_fen_str(None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Knight::calc_movements(&position, position.wn, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["a3", "c3", "d2", "e2", "f3", "h3"],
            "b1a3 b1c3 g1f3 g1h3"
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Knight::calc_movements(&position, position.bn, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["a6", "c6", "d7", "e7", "f6", "h6"],
            "b8a6 b8c6 g8f6 g8h6"
        );


        // 2. Typical position with no pins
        let (_, mut position, mut move_list, _, _) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/QN3N1P/P1P2PP1/3RKB1R w KQkq - 1 2"));
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Knight::calc_movements(&position, position.wn, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["a5", "c5", "d4", "d2", "c1", "a1", "e1", "g1", "h4", "h2", "e5", "g5"],
            "b3a1 b3a5 b3c1 b3c5 b3d2 b3d4 f3d2 f3d4 f3g1 f3h2 f3h4"
        );

        LegalMovesTestHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            Knight::calc_movements(&position, position.bn, &mut move_list, 0, &mut king_attack_analyzer),
            &move_list,
            vec!["e8", "g8", "d7", "h7", "g4", "e4", "d5", "h5"],
            "f6d5 f6d7 f6e4 f6e8 f6h5"
        );
    }

    #[test]
    fn test_calc_knight_movements_benchmark() {
        // let iterations = 40000000;   // currently about 8.5s after calculating and storing pawn moves only
        let iterations = 100;

        let position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/QN3N1P/P1P2PP1/3RKB1R w KQkq b6 1 2"), true).unwrap();
        let mut king_attack_analyzer = KingAttackRayAnalyzer::default();
        let mut move_list = GameMoveList::default();
        let before = Instant::now();
        for _ in 0..iterations {
            move_list.clear();
            Knight::calc_movements(&position, position.wn, &mut move_list, 0, &mut king_attack_analyzer);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
    }
}