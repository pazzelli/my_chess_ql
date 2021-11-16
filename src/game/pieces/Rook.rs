use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::movegenerator::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct Rook {

}

impl Rook {
    #[inline(always)]
    fn add_rook_movement(move_list: &mut GameMoveList, source_square: u8, mut target_squares: u64, is_capture: bool) {
        while target_squares > 0 {
            // trailing_zeros() gives square index from 0..63
            move_list.add_move(PieceType::ROOK, source_square, target_squares.trailing_zeros() as u8, is_capture, PieceType::NONE);
            target_squares &= target_squares - 1;
        }
    }
}

impl Piece for Rook {
    fn calc_attacked_squares(position: &Position, mut piece_pos: u64, _player: PlayerColour) -> u64 {
        let mut rook_attacks = 0u64;

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            let (rank_index, file_index) = PositionHelper::rank_and_file_from_index(sq_ind as u8);

            rook_attacks |= Rook::calc_rank_attacks(&position, sq_ind, file_index, RANKS[rank_index as usize]);
            rook_attacks |= Rook::calc_file_or_diagonal_attacks(&position, sq_ind, FILES[file_index as usize]);

            piece_pos &= piece_pos - 1;
        }

        rook_attacks
    }

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by the rook
    //  movementSquares = squares where the rook can either move or capture a piece
    #[inline(always)]
    fn calc_movements(position: &Position, mut piece_pos: u64, move_list: &mut GameMoveList, _enemy_attacked_squares: Option<u64>) -> (u64, u64) {
        let attacked_squares = Rook::calc_attacked_squares(position, piece_pos, if position.white_to_move {PlayerColour::WHITE} else {PlayerColour::BLACK});

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;

            let capture_squares = attacked_squares & position.enemy_occupancy;
            let non_capture_squares = attacked_squares & position.non_occupancy;
            Rook::add_rook_movement(move_list, sq_ind as u8, capture_squares, true);
            Rook::add_rook_movement(move_list, sq_ind as u8, non_capture_squares, false);

            piece_pos &= piece_pos - 1;
        }

        (attacked_squares, attacked_squares & !position.friendly_occupancy)
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