use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;
use crate::game::positionanalyzer::*;


pub struct Knight {

}

impl Piece for Knight {
    fn get_piece_type() -> PieceType { PieceType::KNIGHT }

    fn calc_attacked_squares(_position: &Position, mut piece_pos: u64, _player: &PlayerColour, enemy_king_pos: u64) -> (u64, KingAttackRayAnalysis) {
        let mut knight_attacks: u64 = 0;
        let mut king_check_board = 0u64;

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            knight_attacks |= KNIGHT_ATTACKS[sq_ind];

            king_check_board |= SINGLE_BITBOARDS[sq_ind] & PositionHelper::bool_to_bitboard(KNIGHT_ATTACKS[sq_ind] & enemy_king_pos > 0);
            piece_pos &= piece_pos - 1;
        }

        (knight_attacks, KingAttackRayAnalysis(0u64, king_check_board, (king_check_board > 0) as u8, false))
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;

    #[test]
    fn test_calc_knight_movements() {
        let mut move_list = GameMoveList::default();

        // 1. Starting position
        let mut position = Position::from_fen(None).unwrap();
        let (attacked_squares, movement_squares) = Knight::calc_movements(&position, position.wn, &mut move_list, None);

        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "c3", "d2", "e2", "f3", "h3"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "c3", "f3", "h3"]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let (attacked_squares, movement_squares) = Knight::calc_movements(&position, position.bn, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "c6", "d7", "e7", "f6", "h6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "c6", "f6", "h6"]));


        // 2. Typical position with no pins
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/QN3N1P/P1P2PP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.clear();
        let (attacked_squares, movement_squares) = Knight::calc_movements(&position, position.wn, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a5", "c5", "d4", "d2", "c1", "a1", "e1", "g1", "h4", "h2", "e5", "g5"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a5", "c5", "d4", "d2", "c1", "a1", "g1", "h2", "h4"]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let (attacked_squares, movement_squares) = Knight::calc_movements(&position, position.bn, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["e8", "g8", "d7", "h7", "g4", "e4", "d5", "h5"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["e8", "d7", "e4", "d5", "h5"]));
    }

    #[test]
    fn test_calc_knight_movements_benchmark() {
        // let iterations = 40000000;   // currently about 8.5s after calculating and storing pawn moves only
        let iterations = 100;

        let position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/QN3N1P/P1P2PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        let mut move_list = GameMoveList::default();
        let before = Instant::now();
        for _ in 0..iterations {
            move_list.clear();
            Knight::calc_movements(&position, position.wn, &mut move_list, None);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
    }
}