use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::positionanalyzer::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct Rook {

}

impl Piece for Rook {
    fn get_piece_type() -> PieceType { PieceType::ROOK }

    fn calc_attacked_squares(position: &Position, mut piece_pos: u64, _player: &PlayerColour, enemy_king_pos: u64) -> (u64, KingAttackRayAnalysis) {
        let mut rook_attacks = 0u64;
        let mut possible_king_attacks = KingAttackRayAnalysis::default();

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;

            rook_attacks |= Rook::calc_rank_attacks(&position, sq_ind, RANKS[sq_ind], enemy_king_pos);
            rook_attacks |= Rook::calc_file_or_diagonal_attacks(&position, sq_ind, FILES[sq_ind], enemy_king_pos);

            if RANKS[sq_ind] & enemy_king_pos > 0 {
                let king_sq = enemy_king_pos.trailing_zeros() as usize;
                possible_king_attacks += Rook::analyze_king_attack_ray(position, SINGLE_BITBOARDS[sq_ind], ATTACK_RAYS[(sq_ind << 6) + king_sq], true, enemy_king_pos);
            }

            if FILES[sq_ind] & enemy_king_pos > 0 {
                let king_sq = enemy_king_pos.trailing_zeros() as usize;
                possible_king_attacks += Rook::analyze_king_attack_ray(position, SINGLE_BITBOARDS[sq_ind], ATTACK_RAYS[(sq_ind << 6) + king_sq], false, enemy_king_pos);
            }

            piece_pos &= piece_pos - 1;
        }

        (rook_attacks, possible_king_attacks)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;

    #[test]
    fn test_calc_rook_movements() {
        let mut move_list = GameMoveList::default();

        // 1. Starting position
        let mut position = Position::from_fen(Some("8/8/3K4/8/8/8/Pk1r4/R2N4 w - - 1 2")).unwrap();
        let (attacked_squares, movement_squares) = Rook::calc_movements(&position, position.wr, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["b1", "c1", "d1", "a2"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["b1", "c1"]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let (attacked_squares, movement_squares) = Rook::calc_movements(&position, position.br, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "c2", "b2", "e2", "f2", "h2", "g2", "d3", "d4", "d5", "d6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["d1", "c2", "e2", "f2", "h2", "g2", "d3", "d4", "d5", "d6"]));


        // 2. Position with friendly pieces in the way, one pawn to capture, one empty square that is enemy-controlled
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2pP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.clear();
        let (attacked_squares, movement_squares) = Rook::calc_movements(&position, position.wr, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a1", "b1", "c1", "d2", "d3", "d4", "d5", "e1", "f1", "g1", "h2", "h3"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a1", "b1", "c1", "d2", "d3", "d4", "g1", "h2", "h3"]));

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let (attacked_squares, movement_squares) = Rook::calc_movements(&position, position.br, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a7", "b8", "c8", "d8", "e8", "f7", "g8"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["b8", "c8", "e8"]));
    }
}