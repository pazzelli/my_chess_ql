use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::positionanalyzer::*;
use crate::game::pieces::piece::*;
use crate::game::pieces::bishop::Bishop;
use crate::game::pieces::rook::Rook;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct Queen {

}

impl Piece for Queen {
    fn get_piece_type() -> PieceType { PieceType::QUEEN }

    // TODO: come up with a more efficient implementation for queen movements (using SIMD?)
    fn calc_attacked_squares(position: &Position, piece_pos: u64, player: &PlayerColour, enemy_king_pos: u64) -> (u64, KingAttackRayAnalysis) {
        let (diagonal_attack_squares, mut king_attacks) = Bishop::calc_attacked_squares(position, piece_pos, player, enemy_king_pos);
        let (non_diagonal_attack_squares, non_diagonal_king_attacks) = Rook::calc_attacked_squares(position, piece_pos, player, enemy_king_pos);

        king_attacks += non_diagonal_king_attacks;
        (diagonal_attack_squares | non_diagonal_attack_squares, king_attacks)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;

    #[test]
    fn test_calc_queen_movements() {
        let mut move_list = GameMoveList::default();

        // 1. Starting position
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2pP1/3RKB1R w - - 1 2")).unwrap();
        let (attacked_squares, movement_squares) = Queen::calc_movements(&position, position.wq, &mut move_list, None);
        // println!("attacked: {:?}", PositionHelper::algebraic_from_bitboard(attacked_squares));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["c1", "a2", "b2", "b3", "c3", "d3", "e3", "f3", "g3", "a4", "b4", "a5", "c5", "a6", "a7"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["c1", "b2", "b3", "c3", "d3", "e3", "f3", "g3", "a4", "b4", "a5", "c5", "a6", "a7"]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let (attacked_squares, movement_squares) = Queen::calc_movements(&position, position.bq, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a8", "f8", "e7", "b8", "c8", "e8", "a5", "b6", "c7", "d7", "d6", "d5"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["b8", "c8", "e8", "a5", "b6", "c7", "d7", "d6", "d5"]));
    }
}