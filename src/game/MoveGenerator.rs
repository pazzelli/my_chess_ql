use bitintr::Tzcnt;
use crate::uci::engine::game_move::*;
use crate::uci::engine::game_move_list::*;
use crate::uci::engine::constants::*;
use crate::uci::engine::position::Position;
use crate::uci::engine::position_helper::PositionHelper;
// use core_simd::*;

pub struct PositionAnalyzer {

}

impl PositionAnalyzer {
    // TODO:  **** test this!!! **** then do a git check-in
    #[inline(always)]
    fn add_pawn_movements(move_list: &mut GameMoveList, mut squares: u64, source_square_offset: i8, is_capture: bool) {
        while squares > 0 {
            // trailing_zeros() gives square index from 0..63
            move_list.add_pawn_movement(squares.trailing_zeros() as u8, source_square_offset, is_capture);
            squares &= squares - 1;
        }
    }

    // Returns (attackedSquares, movementSquares) where attackedSquares = squares controlled by pawns
    //  movementSquares = squares where pawns can either move or capture a piece
    #[inline(always)]
    pub fn calc_pawn_movements(position: &Position, move_list: &mut GameMoveList) -> (u64, u64) {
        // TODO: add pinned piece support

        // TODO: add back analysis for black after ensuring white's is correct
        // if position.white_to_move {
            let possible_capture_squares = position.black_occupancy | position.en_passant_sq;

            let left_attacked = (position.wp & !A_FILE) << 7;
            let right_attacked = (position.wp & !H_FILE) << 9;
            let attacked_squares = left_attacked | right_attacked;

            let capture_squares = attacked_squares & possible_capture_squares;
            PositionAnalyzer::add_pawn_movements(move_list, left_attacked & capture_squares, -7, true);
            PositionAnalyzer::add_pawn_movements(move_list, right_attacked & capture_squares, -9, true);

            let forward_one_square_moves = (position.wp << 8) & position.non_occupancy;
            let forward_two_square_moves = ((position.wp & RANK_2) << 16) & (forward_one_square_moves << 8) & position.non_occupancy;
            let movement_squares = capture_squares | forward_one_square_moves | forward_two_square_moves;
            PositionAnalyzer::add_pawn_movements(move_list, forward_one_square_moves, -8, false);
            PositionAnalyzer::add_pawn_movements(move_list, forward_two_square_moves, -16, false);

            (attacked_squares, movement_squares)



            // // White's pawn moves
            // let forward_one_square_moves = (position.wp << 8) & position.non_occupancy;
            // (
            //     // Left-side pawn attack squares
            //     ((position.wp & !A_FILE) << 7)
            //
            //     // Right-side pawn attack squares
            //     | ((position.wp & !H_FILE) << 9)
            // )
            // // can only capture enemy pieces or on en-passant square
            // & (position.black_occupancy | position.en_passant_sq)
            // | (
            //     // Forward-moving squares (single square)
            //     forward_one_square_moves
            //     // Forward-moving squares (two squares)
            //     | (((position.wp & RANK_2) << 16) & (forward_one_square_moves << 8) & position.non_occupancy)
            // )

        // } else {
        //     // Black's pawn moves
        //     let forward_one_square_moves = (position.bp >> 8) & position.non_occupancy;
        //     (
        //         // Left-side pawn attack squares
        //         ((position.bp & !H_FILE) >> 7)
        //
        //             // Right-side pawn attack squares
        //             | ((position.bp & !A_FILE) >> 9)
        //     )
        //     // can only capture enemy pieces or on en-passant square
        //     & (position.white_occupancy | position.en_passant_sq)   // can only capture enemy pieces
        //     | (
        //         // Forward-moving squares (single square)
        //         forward_one_square_moves
        //         // Forward-moving squares (two squares)
        //         | (((position.bp & RANK_7) >> 16) & (forward_one_square_moves >> 8) & position.non_occupancy)
        //     )
        // }
    }

    // pub fn get_legal_moves(position: &Position) -> GameMoveList {
    //     let mut move_list = GameMoveList::default();
    //
    //     // let positiveRayAttacks: u64 = occupancy  ^ (occupancy - 2s);
    //     let (pawn_attacks, pawn_movements) = PositionAnalyzer::calc_pawn_movements(position, &mut move_list);
    //
    //     move_list
    // }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;

    fn add_pawn_moves_to_list(move_list: &mut GameMoveList, source_squares: &[u8], target_squares: &[u8], promotion_piece: Vec<Option<Piece>>, is_capture: &[bool]) {
        for i in 0..source_squares.len() {
            move_list.move_list.push(GameMove{
                piece: Piece::PAWN,
                source_square: source_squares[i],
                target_square: target_squares[i],
                promotion_piece: promotion_piece[i],
                is_capture: is_capture[i]
            })
        }
    }

    #[test]
    fn test_calc_pawn_movements() {
        // TODO: add support for pinned pieces
        let mut move_list = GameMoveList::default();

        // 1. Starting position
        let mut position = Position::from_fen(None).unwrap();
        let (attacked_squares, movement_squares) = PositionAnalyzer::calc_pawn_movements(&position, &mut move_list);

        // println!("{}", move_list.move_list[0]);

        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "a4", "b3", "b4", "c3", "c4", "d3", "d4", "e3", "e4", "f3", "f4", "g3", "g4", "h3", "h4"]));
        // let mut target_move_list = GameMoveList::default();
        // add_pawn_moves_to_list(&mut target_move_list,
        //                        [30, 41, 42, 56, ])
        // assert_eq!(move_list, target_move_list);

        position.white_to_move = false;
        move_list.move_list.clear();
        let (attacked_squares, movement_squares) = PositionAnalyzer::calc_pawn_movements(&position, &mut move_list);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "a5", "b6", "b5", "c6", "c5", "d6", "d5", "e6", "e5", "f6", "f5", "g6", "g5", "h6", "h5"]));


        // 2. Typical position with no en-passant or pins
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/3PP1B1/51b1/Q4N1P/P1P2PP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.move_list.clear();
        let (attacked_squares, movement_squares) = PositionAnalyzer::calc_pawn_movements(&position, &mut move_list);
        // let x = PositionAnalyzer::calc_pawn_movements(&position);
        // println!("{:x}", x);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["b3", "d3", "e3", "f3", "g3", "h3", "g4", "c6", "d6", "e6", "f6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["c3", "c4", "c6", "d6", "e6", "f6", "g3", "g4", "h4"]));

        position.white_to_move = false;
        move_list.move_list.clear();
        let (attacked_squares, movement_squares) = PositionAnalyzer::calc_pawn_movements(&position, &mut move_list);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "c6", "d6", "e6", "f6", "g6", "b5", "d5", "f5", "h5"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "a5", "b6", "b5", "c5", "d5", "e6", "h6", "h5"]));


        // 3. Position including en-passant on b6 square
        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        move_list.move_list.clear();
        let (attacked_squares, movement_squares) = PositionAnalyzer::calc_pawn_movements(&position, &mut move_list);
        // println!("{:x}", PositionAnalyzer::calc_pawn_movements(&position));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["b6", "c6", "d6", "e6", "f6", "a8", "c8", "e3", "f3", "g3", "h3", "g4"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "a8", "b8", "c6", "d6", "e6", "f6", "g3", "g4", "h4"]));
    }

    #[test]
    fn test_calc_pawn_movements_benchmark() {
        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        let mut move_list = GameMoveList::default();
        let before = Instant::now();
        for _ in 0..40000000 {
            move_list.move_list.clear();
            // let mut move_list = GameMoveList::default();
            PositionAnalyzer::calc_pawn_movements(&position, &mut move_list);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
    }

    #[test]
    fn test_add_pawn_movements() {
        let mut move_list = GameMoveList::default();
        PositionAnalyzer::add_pawn_movements(&mut move_list, 0xff0000u64, -8, false);
        assert_eq!(
            format!("{:?}", move_list.move_list),
            "[GameMove { piece: PAWN, source_square: 8, target_square: 16, promotion_piece: None, is_capture: false }, GameMove { piece: PAWN, source_square: 9, target_square: 17, promotion_piece: None, is_capture: false }, GameMove { piece: PAWN, source_square: 10, target_square: 18, promotion_piece: None, is_capture: false }, GameMove { piece: PAWN, source_square: 11, target_square: 19, promotion_piece: None, is_capture: false }, GameMove { piece: PAWN, source_square: 12, target_square: 20, promotion_piece: None, is_capture: false }, GameMove { piece: PAWN, source_square: 13, target_square: 21, promotion_piece: None, is_capture: false }, GameMove { piece: PAWN, source_square: 14, target_square: 22, promotion_piece: None, is_capture: false }, GameMove { piece: PAWN, source_square: 15, target_square: 23, promotion_piece: None, is_capture: false }]"
        );


        move_list.move_list.clear();
        PositionAnalyzer::add_pawn_movements(
            &mut move_list,
            PositionHelper::bitboard_from_algebraic(vec!["a8", "d8", "a6", "d3", "h7", "g4"]),
            -8,
            true
        );
        assert_eq!(
            format!("{:?}", move_list.move_list),
            "[GameMove { piece: PAWN, source_square: 11, target_square: 19, promotion_piece: None, is_capture: true }, GameMove { piece: PAWN, source_square: 22, target_square: 30, promotion_piece: None, is_capture: true }, GameMove { piece: PAWN, source_square: 32, target_square: 40, promotion_piece: None, is_capture: true }, GameMove { piece: PAWN, source_square: 47, target_square: 55, promotion_piece: None, is_capture: true }, GameMove { piece: PAWN, source_square: 48, target_square: 56, promotion_piece: Some(KNIGHT), is_capture: true }, GameMove { piece: PAWN, source_square: 48, target_square: 56, promotion_piece: Some(QUEEN), is_capture: true }, GameMove { piece: PAWN, source_square: 51, target_square: 59, promotion_piece: Some(KNIGHT), is_capture: true }, GameMove { piece: PAWN, source_square: 51, target_square: 59, promotion_piece: Some(QUEEN), is_capture: true }]"
        );
    }

    #[test]
    fn test_bit_scan_speed() {
        let before = Instant::now();
        for _ in 0..40000000 {
            assert_eq!(0xff0000u64.tzcnt() as u8, 16_u8);
        }
        println!("tzcnt - Elapsed time: {:.2?}", before.elapsed());

        let before = Instant::now();
        for _ in 0..40000000 {
            assert_eq!(0xff0000u64.trailing_zeros() as u8, 16_u8);
        }
        println!("trailing_zeros - Elapsed time: {:.2?}", before.elapsed());

    }
}