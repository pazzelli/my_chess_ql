use crate::constants::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;


pub struct Knight {

}

impl Knight {
    #[inline(always)]
    fn add_knight_movement(move_list: &mut GameMoveList, source_square: u8, mut target_squares: u64, is_capture: bool) {
        while target_squares > 0 {
            // trailing_zeros() gives square index from 0..63
            move_list.add_move(PieceType::KNIGHT, source_square, target_squares.trailing_zeros() as u8, is_capture, PieceType::NONE);
            target_squares &= target_squares - 1;
        }
    }
}

impl Piece for Knight {
    fn calc_attacked_squares(_position: &Position, mut piece_pos: u64, _player: &PlayerColour) -> u64 {
        let mut knight_attacks: u64 = 0;
        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            knight_attacks |= KNIGHT_ATTACKS[sq_ind];
            piece_pos &= piece_pos - 1;
        }

        knight_attacks
    }

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by knights
    //  movementSquares = squares where knights can either move or capture a piece
    #[inline(always)]
    fn calc_movements(position: &Position, mut piece_pos: u64, move_list: &mut GameMoveList, _enemy_attacked_squares: Option<u64>) -> (u64, u64) {
        // TODO: add pinned piece support

        let mut knight_attacks = 0u64;
        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            knight_attacks |= KNIGHT_ATTACKS[sq_ind];

            let cur_knight_captures: u64 = KNIGHT_ATTACKS[sq_ind] & position.enemy_occupancy;
            let cur_knight_non_captures: u64 = KNIGHT_ATTACKS[sq_ind] & position.non_occupancy;

            Knight::add_knight_movement(move_list, sq_ind as u8, cur_knight_captures, true);
            Knight::add_knight_movement(move_list, sq_ind as u8, cur_knight_non_captures, false);

            piece_pos &= piece_pos - 1;
        }

        (knight_attacks, knight_attacks & !position.friendly_occupancy)
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
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: KNIGHT, source_square: \"b1\", target_square: \"a3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b1\", target_square: \"c3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"g1\", target_square: \"f3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"g1\", target_square: \"h3\", promotion_piece: NONE, is_capture: false }]"
        // );

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let (attacked_squares, movement_squares) = Knight::calc_movements(&position, position.bn, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "c6", "d7", "e7", "f6", "h6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "c6", "f6", "h6"]));
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: KNIGHT, source_square: \"b8\", target_square: \"a6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b8\", target_square: \"c6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"g8\", target_square: \"f6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"g8\", target_square: \"h6\", promotion_piece: NONE, is_capture: false }]"
        // );

        // 2. Typical position with no pins
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/QN3N1P/P1P2PP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.clear();
        let (attacked_squares, movement_squares) = Knight::calc_movements(&position, position.wn, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a5", "c5", "d4", "d2", "c1", "a1", "e1", "g1", "h4", "h2", "e5", "g5"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a5", "c5", "d4", "d2", "c1", "a1", "g1", "h2", "h4"]));
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"c5\", promotion_piece: NONE, is_capture: true }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"a1\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"c1\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"d2\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"d4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"a5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"g1\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"d2\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"h2\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"d4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"h4\", promotion_piece: NONE, is_capture: false }]"
        // );

        position.white_to_move = false;
        position.update_occupancy();
        move_list.clear();
        let (attacked_squares, movement_squares) = Knight::calc_movements(&position, position.bn, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["e8", "g8", "d7", "h7", "g4", "e4", "d5", "h5"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["e8", "d7", "e4", "d5", "h5"]));
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"d5\", promotion_piece: NONE, is_capture: true }, GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"e4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"h5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"d7\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"e8\", promotion_piece: NONE, is_capture: false }]"
        // );
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