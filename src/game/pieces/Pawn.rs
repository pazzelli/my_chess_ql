use crate::constants::*;
use crate::game::pieces::piece::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;
use crate::game::gamemovelist::GameMoveList;

pub struct Pawn {
}

impl Pawn {
    #[inline(always)]
    fn calc_left_right_attacks(position: &Position, player: PlayerColour) -> (u64, u64) {
        // Squares attacked to the forward-left of pawns
        let left_attacked;
        let right_attacked;
        match player {
            PlayerColour::WHITE => {
                left_attacked = (position.wp & !A_FILE) << 7;
                right_attacked = (position.wp & !H_FILE) << 9;
            }
            PlayerColour::BLACK => {
                left_attacked = (position.bp & !H_FILE) >> 7;
                right_attacked = (position.bp & !A_FILE) >> 9;
            }
        }

        (left_attacked, right_attacked)
    }

    #[inline(always)]
    pub fn add_pawn_movement(move_list: &mut GameMoveList, mut squares: u64, source_square_offset: i8, is_capture: bool) {
        while squares > 0 {
            // trailing_zeros() gives square index from 0..63
            let target_square = squares.trailing_zeros() as u8;

            let target_rank = target_square >> 3;
            if target_rank == 0 || target_rank == 7 {
                move_list.add_move(PieceType::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, PieceType::KNIGHT);
                move_list.add_move(PieceType::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, PieceType::BISHOP);
                move_list.add_move(PieceType::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, PieceType::ROOK);
                move_list.add_move(PieceType::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, PieceType::QUEEN);
            } else {
                move_list.add_move(PieceType::PAWN, (target_square as i8 + source_square_offset) as u8, target_square, is_capture, PieceType::NONE);
            }

            squares &= squares - 1;
        }
    }
}

impl Piece for Pawn {
    #[inline(always)]
    fn calc_attacked_squares(position: &Position, player: PlayerColour) -> u64 {
        let (left_attacked, right_attacked) = Pawn::calc_left_right_attacks(position, player);
        left_attacked | right_attacked
    }

    #[inline(always)]
    fn calc_movements(position: &Position, move_list: &mut GameMoveList, _enemy_attacked_squares: Option<u64>) -> (u64, u64) {
        // TODO: add pinned piece support

        let mut source_sq_offset_multiplier = 1i8;
        let possible_capture_squares;

        let (left_attacked, right_attacked) = Pawn::calc_left_right_attacks(position, if position.white_to_move {PlayerColour::WHITE} else {PlayerColour:: BLACK});
        let forward_one_square_moves;
        let forward_two_square_moves;

        if position.white_to_move {
            // Calculations for white
            source_sq_offset_multiplier = -1;
            possible_capture_squares = position.black_occupancy | position.en_passant_sq;

            forward_one_square_moves = (position.wp << 8) & position.non_occupancy;
            forward_two_square_moves = ((position.wp & RANK_2) << 16) & (forward_one_square_moves << 8) & position.non_occupancy;

        } else {
            // Calculations for black
            possible_capture_squares = position.white_occupancy | position.en_passant_sq;

            forward_one_square_moves = (position.bp >> 8) & position.non_occupancy;
            forward_two_square_moves = ((position.bp & RANK_7) >> 16) & (forward_one_square_moves >> 8) & position.non_occupancy;
        }

        let attacked_squares = left_attacked | right_attacked;

        let capture_squares = attacked_squares & possible_capture_squares;
        Pawn::add_pawn_movement(move_list, left_attacked & capture_squares, source_sq_offset_multiplier * 7, true);
        Pawn::add_pawn_movement(move_list, right_attacked & capture_squares, source_sq_offset_multiplier * 9, true);

        let movement_squares = capture_squares | forward_one_square_moves | forward_two_square_moves;
        Pawn::add_pawn_movement(move_list, forward_one_square_moves, source_sq_offset_multiplier * 8, false);
        Pawn::add_pawn_movement(move_list, forward_two_square_moves, source_sq_offset_multiplier * 16, false);

        (attacked_squares, movement_squares)
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
        let (attacked_squares, movement_squares) = Pawn::calc_movements(&position, &mut move_list, None);

        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a3", "a4", "b3", "b4", "c3", "c4", "d3", "d4", "e3", "e4", "f3", "f4", "g3", "g4", "h3", "h4"]));
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: PAWN, source_square: \"a2\", target_square: \"a3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b2\", target_square: \"b3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e2\", target_square: \"e3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f2\", target_square: \"f3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h2\", target_square: \"h3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a2\", target_square: \"a4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b2\", target_square: \"b4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e2\", target_square: \"e4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f2\", target_square: \"f4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h2\", target_square: \"h4\", promotion_piece: NONE, is_capture: false }]"
        // );

        position.white_to_move = false;
        move_list.clear();
        let (attacked_squares, movement_squares) = Pawn::calc_movements(&position, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "a5", "b6", "b5", "c6", "c5", "d6", "d5", "e6", "e5", "f6", "f5", "g6", "g5", "h6", "h5"]));
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c7\", target_square: \"c6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e7\", target_square: \"e6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f7\", target_square: \"f6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g7\", target_square: \"g6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h7\", target_square: \"h6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c7\", target_square: \"c5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e7\", target_square: \"e5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f7\", target_square: \"f5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g7\", target_square: \"g5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h7\", target_square: \"h5\", promotion_piece: NONE, is_capture: false }]"
        // );


        // 2. Typical position with no en-passant or pins
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/Q4N1P/P1P2PP1/3RKB1R w KQkq - 1 2")).unwrap();
        move_list.clear();
        let (attacked_squares, movement_squares) = Pawn::calc_movements(&position, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["b3", "d3", "e3", "f3", "g3", "h3", "g4", "c6", "d6", "e6", "f6"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["c3", "c4", "c6", "d6", "e6", "f6", "g3", "g4", "h4"]));
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: PAWN, source_square: \"h3\", target_square: \"g4\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"d5\", target_square: \"c6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"e5\", target_square: \"f6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h3\", target_square: \"h4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d5\", target_square: \"d6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e5\", target_square: \"e6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c4\", promotion_piece: NONE, is_capture: false }]"
        // );

        position.white_to_move = false;
        move_list.clear();
        let (attacked_squares, movement_squares) = Pawn::calc_movements(&position, &mut move_list, None);
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "c6", "d6", "e6", "f6", "g6", "b5", "d5", "f5", "h5", "b4", "d4"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "a5", "b6", "b5", "c4", "d5", "e6", "h6", "h5"]));
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: PAWN, source_square: \"c6\", target_square: \"d5\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"c5\", target_square: \"c4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e7\", target_square: \"e6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h7\", target_square: \"h6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b5\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h7\", target_square: \"h5\", promotion_piece: NONE, is_capture: false }]"
        // );


        // 3. Position including en-passant on b6 square
        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        move_list.clear();
        let (attacked_squares, movement_squares) = Pawn::calc_movements(&position, &mut move_list, None);
        // println!("{:x}", PositionAnalyzer::calc_pawn_movements(&position));
        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(vec!["b6", "c6", "d6", "e6", "f6", "a8", "c8", "e3", "f3", "g3", "h3", "g4"]));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(vec!["a6", "b6", "a8", "b8", "c6", "d6", "e6", "f6", "g3", "g4", "h4"]));
        // assert_eq!(
        //     format!("{:?}", &move_list.move_list[0..move_list.list_len]),
        //     "[GameMove { piece: PAWN, source_square: \"h3\", target_square: \"g4\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"c5\", target_square: \"b6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"d5\", target_square: \"c6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"a8\", promotion_piece: KNIGHT, is_capture: true }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"a8\", promotion_piece: BISHOP, is_capture: true }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"a8\", promotion_piece: ROOK, is_capture: true }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"a8\", promotion_piece: QUEEN, is_capture: true }, GameMove { piece: PAWN, source_square: \"a5\", target_square: \"b6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"e5\", target_square: \"f6\", promotion_piece: NONE, is_capture: true }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h3\", target_square: \"h4\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"a5\", target_square: \"a6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d5\", target_square: \"d6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e5\", target_square: \"e6\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b8\", promotion_piece: KNIGHT, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b8\", promotion_piece: BISHOP, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b8\", promotion_piece: ROOK, is_capture: false }, GameMove { piece: PAWN, source_square: \"b7\", target_square: \"b8\", promotion_piece: QUEEN, is_capture: false }]"
        // );
    }

    #[test]
    fn test_calc_pawn_movements_benchmark() {
        // let iterations = 40000000;   // currently about 8.5s after calculating and storing pawn moves only
        let iterations = 100;

        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        let mut move_list = GameMoveList::default();
        let before = Instant::now();
        for _ in 0..iterations {
            move_list.clear();
            Pawn::calc_movements(&position, &mut move_list, None);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
    }

    #[test]
    fn test_add_pawn_movements() {
        let mut move_list = GameMoveList::default();
        Pawn::add_pawn_movement(&mut move_list, 0xff0000u64, -8, false);
        assert_eq!(
            format!("{:?}", &move_list.move_list[0..move_list.list_len]),
            "[GameMove { piece: PAWN, source_square: \"a2\", target_square: \"a3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"b2\", target_square: \"b3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"e2\", target_square: \"e3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"f2\", target_square: \"f3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false }, GameMove { piece: PAWN, source_square: \"h2\", target_square: \"h3\", promotion_piece: NONE, is_capture: false }]"
        );


        move_list.clear();
        Pawn::add_pawn_movement(
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
}