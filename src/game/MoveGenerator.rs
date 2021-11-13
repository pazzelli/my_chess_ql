use bitintr::Tzcnt;
use crate::uci::engine::game_move::*;
use crate::uci::engine::game_move_list::*;
use crate::uci::engine::constants::*;
use crate::uci::engine::constants::Piece::NONE;
use crate::uci::engine::position::Position;
use crate::uci::engine::position_helper::PositionHelper;
// use core_simd::*;

pub struct MoveGenerator {

}

impl MoveGenerator {
    #[inline(always)]
    pub fn add_pawn_movement(move_list: &mut GameMoveList, target_square: u8, source_square_offset: i8, is_capture: bool) {
        let target_rank = target_square >> 3;
        if target_rank == 0 || target_rank == 7 {
            move_list.add_move(Piece::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, Piece::KNIGHT);
            move_list.add_move(Piece::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, Piece::BISHOP);
            move_list.add_move(Piece::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, Piece::ROOK);
            move_list.add_move(Piece::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, Piece::QUEEN);
        } else {
            move_list.add_move(Piece::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, Piece::NONE);
        }
    }

    #[inline(always)]
    fn add_pawn_movements(move_list: &mut GameMoveList, mut squares: u64, source_square_offset: i8, is_capture: bool) {
        while squares > 0 {
            // trailing_zeros() gives square index from 0..63
            MoveGenerator::add_pawn_movement(move_list, squares.trailing_zeros() as u8, source_square_offset, is_capture);
            squares &= squares - 1;
        }
    }

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by pawns
    //  movementSquares = squares where pawns can either move or capture a piece
    #[inline(always)]
    pub fn calc_pawn_movements(position: &Position, move_list: &mut GameMoveList) -> (u64, u64) {
        // TODO: add pinned piece support

        let mut source_sq_offset_multiplier = 1i8;
        let possible_capture_squares;

        let left_attacked;
        let right_attacked;
        let forward_one_square_moves;
        let forward_two_square_moves;

        if position.white_to_move {
            // Calculations for white
            source_sq_offset_multiplier = -1;
            possible_capture_squares = position.black_occupancy | position.en_passant_sq;

            left_attacked = (position.wp & !A_FILE) << 7;
            right_attacked = (position.wp & !H_FILE) << 9;

            forward_one_square_moves = (position.wp << 8) & position.non_occupancy;
            forward_two_square_moves = ((position.wp & RANK_2) << 16) & (forward_one_square_moves << 8) & position.non_occupancy;

        } else {
            // Calculations for black
            possible_capture_squares = position.white_occupancy | position.en_passant_sq;

            left_attacked = (position.bp & !H_FILE) >> 7;
            right_attacked = (position.bp & !A_FILE) >> 9;

            forward_one_square_moves = (position.bp >> 8) & position.non_occupancy;
            forward_two_square_moves = ((position.bp & RANK_7) >> 16) & (forward_one_square_moves >> 8) & position.non_occupancy;
        }

        let attacked_squares = left_attacked | right_attacked;

        let capture_squares = attacked_squares & possible_capture_squares;
        MoveGenerator::add_pawn_movements(move_list, left_attacked & capture_squares, source_sq_offset_multiplier * 7, true);
        MoveGenerator::add_pawn_movements(move_list, right_attacked & capture_squares, source_sq_offset_multiplier * 9, true);

        let movement_squares = capture_squares | forward_one_square_moves | forward_two_square_moves;
        MoveGenerator::add_pawn_movements(move_list, forward_one_square_moves, source_sq_offset_multiplier * 8, false);
        MoveGenerator::add_pawn_movements(move_list, forward_two_square_moves, source_sq_offset_multiplier * 16, false);

        (attacked_squares, movement_squares)
    }

    #[inline(always)]
    fn add_knight_movements(move_list: &mut GameMoveList, source_square: u8, mut target_squares: u64, is_capture: bool) {
        while target_squares > 0 {
            // trailing_zeros() gives square index from 0..63
            move_list.add_move(Piece::KNIGHT, source_square, target_squares.trailing_zeros() as u8, is_capture, NONE);
            target_squares &= target_squares - 1;
        }
    }

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by knights
    //  movementSquares = squares where knights can either move or capture a piece
    #[inline(always)]
    pub fn calc_knight_movements(position: &Position, move_list: &mut GameMoveList) -> (u64, u64) {
        // TODO: add pinned piece support

        let mut knight_attacks = 0u64;

        let mut knight_pos;
        let enemy_occupancy;
        let friendly_occupancy;

        if position.white_to_move {
            knight_pos = position.wn;
            enemy_occupancy = position.black_occupancy;
            friendly_occupancy = position.white_occupancy;
        } else {
            knight_pos = position.bn;
            enemy_occupancy = position.white_occupancy;
            friendly_occupancy = position.black_occupancy;
        }

        while knight_pos > 0 {
            let sq_ind: usize = knight_pos.trailing_zeros() as usize;
            knight_attacks |= KNIGHT_ATTACKS[sq_ind];

            // let mut cur_knight_attacks: u64 = KNIGHT_ATTACKS[sq_ind];
            let cur_knight_captures: u64 = KNIGHT_ATTACKS[sq_ind] & enemy_occupancy;
            let cur_knight_non_captures: u64 = KNIGHT_ATTACKS[sq_ind] & position.non_occupancy;

            MoveGenerator::add_knight_movements(move_list, sq_ind as u8, cur_knight_captures, true);
            MoveGenerator::add_knight_movements(move_list, sq_ind as u8, cur_knight_non_captures, false);

            knight_pos &= knight_pos - 1;
        }

        (knight_attacks, knight_attacks & !friendly_occupancy)
    }

    pub fn calc_legal_moves(position: &Position, move_list: &mut GameMoveList) {
        // let mut move_list = GameMoveList::default();

        // let positiveRayAttacks: u64 = occupancy  ^ (occupancy - 2s);
        let (_pawn_attacks, _pawn_movements) = MoveGenerator::calc_pawn_movements(position, move_list);
        let (_knight_attacks, _knight_movements) = MoveGenerator::calc_knight_movements(position, move_list);

        // move_list
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;

    #[test]
    fn test_calc_pawn_movements() {
        // TODO: add support for pinned pieces
        let mut move_list = GameMoveList::default();

        // 1. Starting position
        let mut position = Position::from_fen(None).unwrap();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_pawn_movements(&position, &mut move_list);

        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "a4", "b3", "b4", "c3", "c4", "d3", "d4", "e3", "e4", "f3", "f4", "g3", "g4", "h3", "h4"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: PAWN, source_square: \"a2\", target_square: \"a3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b2\", target_square: \"b3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e2\", target_square: \"e3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f2\", target_square: \"f3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h2\", target_square: \"h3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a2\", target_square: \"a4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b2\", target_square: \"b4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e2\", target_square: \"e4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f2\", target_square: \"f4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h2\", target_square: \"h4\", promotion_piece: NONE, is_capture: false }]"
        );

        position.white_to_move = false;
        move_list.clear();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_pawn_movements(&position, &mut move_list);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "a5", "b6", "b5", "c6", "c5", "d6", "d5", "e6", "e5", "f6", "f5", "g6", "g5", "h6", "h5"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c7\", target_square: \"c6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e7\", target_square: \"e6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f7\", target_square: \"f6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g7\", target_square: \"g6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h7\", target_square: \"h6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c7\", target_square: \"c5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e7\", target_square: \"e5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f7\", target_square: \"f5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g7\", target_square: \"g5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h7\", target_square: \"h5\", promotion_piece: NONE, is_capture: false }]"
        );


        // 2. Typical position with no en-passant or pins
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/Q4N1P/P1P2PP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.clear();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_pawn_movements(&position, &mut move_list);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["b3", "d3", "e3", "f3", "g3", "h3", "g4", "c6", "d6", "e6", "f6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["c3", "c4", "c6", "d6", "e6", "f6", "g3", "g4", "h4"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: PAWN, source_square: \"h3\", target_square: \"g4\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"d5\", target_square: \"c6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"e5\", target_square: \"f6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h3\", target_square: \"h4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d5\", target_square: \"d6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e5\", target_square: \"e6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c4\", promotion_piece: NONE, is_capture: false }]"
        );

        position.white_to_move = false;
        move_list.clear();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_pawn_movements(&position, &mut move_list);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "c6", "d6", "e6", "f6", "g6", "b5", "d5", "f5", "h5", "b4", "d4"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "a5", "b6", "b5", "c4", "d5", "e6", "h6", "h5"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: PAWN, source_square: \"c6\", target_square: \"d5\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"c5\", target_square: \"c4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e7\", target_square: \"e6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h7\", target_square: \"h6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h7\", target_square: \"h5\", promotion_piece: NONE, is_capture: false }]"
        );


        // 3. Position including en-passant on b6 square
        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        move_list.clear();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_pawn_movements(&position, &mut move_list);
        // println!("{:x}", PositionAnalyzer::calc_pawn_movements(&position));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["b6", "c6", "d6", "e6", "f6", "a8", "c8", "e3", "f3", "g3", "h3", "g4"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "a8", "b8", "c6", "d6", "e6", "f6", "g3", "g4", "h4"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: PAWN, source_square: \"h3\", target_square: \"g4\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"c5\", target_square: \"b6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"d5\", target_square: \"c6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"a8\", promotion_piece: KNIGHT, is_capture: true }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"a8\", promotion_piece: BISHOP, is_capture: true }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"a8\", promotion_piece: ROOK, is_capture: true }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"a8\", promotion_piece: QUEEN, is_capture: true }, GameMove { piece: PAWN, source_square: \"a5\", target_square: \"b6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"e5\", target_square: \"f6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h3\", target_square: \"h4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a5\", target_square: \"a6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d5\", target_square: \"d6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e5\", target_square: \"e6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b8\", promotion_piece: KNIGHT, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b8\", promotion_piece: BISHOP, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b8\", promotion_piece: ROOK, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b8\", promotion_piece: QUEEN, is_capture: false }]"
        );
    }

    #[test]
    fn test_calc_pawn_movements_benchmark() {
        // let iterations = 40000000;   // currently about 8.5s after calculating and storing pawn moves only
        let iterations = 100;

        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        let mut move_list = GameMoveList::default();
        let before = Instant::now();
        for _ in 0..iterations {
            // move_list.move_list.clear();
            move_list.clear();
            MoveGenerator::calc_pawn_movements(&position, &mut move_list);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
    }

    #[test]
    fn test_add_pawn_movements() {
        let mut move_list = GameMoveList::default();
        MoveGenerator::add_pawn_movements(&mut move_list, 0xff0000u64, -8, false);
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: PAWN, source_square: \"a2\", target_square: \"a3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b2\", target_square: \"b3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e2\", target_square: \"e3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f2\", target_square: \"f3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h2\", target_square: \"h3\", promotion_piece: NONE, is_capture: false }]"
        );


        move_list.clear();
        MoveGenerator::add_pawn_movements(
            &mut move_list,
            PositionHelper::bitboard_from_algebraic(vec!["a8", "d8", "a6", "d3", "h7", "g4"]),
            -8,
            true
        );
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d3\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"g3\", target_square: \"g4\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"a5\", target_square: \"a6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"h6\", target_square: \"h7\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a8\", promotion_piece: KNIGHT, is_capture: true }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a8\", promotion_piece: BISHOP, is_capture: true }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a8\", promotion_piece: ROOK, is_capture: true }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a8\", promotion_piece: QUEEN, is_capture: true }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d8\", promotion_piece: KNIGHT, is_capture: true }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d8\", promotion_piece: BISHOP, is_capture: true }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d8\", promotion_piece: ROOK, is_capture: true }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d8\", promotion_piece: QUEEN, is_capture: true }]"
        );
    }

    #[test]
    fn test_calc_knight_movements() {
        let mut move_list = GameMoveList::default();

        // 1. Starting position
        let mut position = Position::from_fen(None).unwrap();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_knight_movements(&position, &mut move_list);

        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "c3", "d2", "e2", "f3", "h3"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "c3", "f3", "h3"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: KNIGHT, source_square: \"b1\", target_square: \"a3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b1\", target_square: \"c3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"g1\", target_square: \"f3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"g1\", target_square: \"h3\", promotion_piece: NONE, is_capture: false }]"
        );

        position.white_to_move = false;
        move_list.clear();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_knight_movements(&position, &mut move_list);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "c6", "d7", "e7", "f6", "h6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "c6", "f6", "h6"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: KNIGHT, source_square: \"b8\", target_square: \"a6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b8\", target_square: \"c6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"g8\", target_square: \"f6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"g8\", target_square: \"h6\", promotion_piece: NONE, is_capture: false }]"
        );

        // 2. Typical position with no pins
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/QN3N1P/P1P2PP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.clear();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_knight_movements(&position, &mut move_list);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a5", "c5", "d4", "d2", "c1", "a1", "e1", "g1", "h4", "h2", "e5", "g5"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a5", "c5", "d4", "d2", "c1", "a1", "g1", "h2", "h4"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"c5\", promotion_piece: NONE, is_capture: true }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"a1\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"c1\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"d2\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"d4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"b3\", target_square: \"a5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"g1\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"d2\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"h2\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"d4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f3\", target_square: \"h4\", promotion_piece: NONE, is_capture: false }]"
        );

        position.white_to_move = false;
        move_list.clear();
        let (attacked_squares, movement_squares) = MoveGenerator::calc_knight_movements(&position, &mut move_list);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["e8", "g8", "d7", "h7", "g4", "e4", "d5", "h5"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["e8", "d7", "e4", "d5", "h5"]));
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"d5\", promotion_piece: NONE, is_capture: true }, GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"e4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"h5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"d7\", promotion_piece: NONE, is_capture: false }, GameMove { piece: KNIGHT, source_square: \"f6\", target_square: \"e8\", promotion_piece: NONE, is_capture: false }]"
        );
    }

    #[test]
    fn test_calc_legal_moves_benchmark() {
        // let iterations = 40000000;   // currently about 8.5s after calculating and storing pawn moves only
        let iterations = 100;

        let mut move_list = GameMoveList::default();
        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        let before = Instant::now();
        for _ in 0..iterations {
            move_list.clear();
            MoveGenerator::calc_legal_moves(&position, &mut move_list);
            // println!("{:?}", MoveGenerator::calc_legal_moves(&position).move_list);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
    }

    // #[test]
    // // Result - trailing_zeros() is faster by about 33%
    // fn test_bit_scan_speed() {
    //     let before = Instant::now();
    //     for _ in 0..40000000 {
    //         assert_eq!(0xff0000u64.tzcnt() as u8, 16_u8);
    //     }
    //     println!("tzcnt - Elapsed time: {:.2?}", before.elapsed());
    //
    //     let before = Instant::now();
    //     for _ in 0..40000000 {
    //         assert_eq!(0xff0000u64.trailing_zeros() as u8, 16_u8);
    //     }
    //     println!("trailing_zeros - Elapsed time: {:.2?}", before.elapsed());
    // }
}