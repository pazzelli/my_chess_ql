use crate::constants::*;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::analysis::positionanalyzer::*;
use crate::game::gamemovelist::*;
use crate::game::pieces::piece::*;
use crate::game::position::*;
use crate::game::positionhelper::*;

pub struct Pawn {
}

impl Pawn {
    // PlayerColour is needed here because the direction of pawn movements differs for each side
    #[inline(always)]
    fn calc_left_right_attacks(position: &Position, player: &PlayerColour, enemy_king_pos: u64) -> (u64, u64, u64) {
        // Squares attacked to the forward-left of pawns
        let left_attacked;
        let right_attacked;
        let mut king_check_board = 0u64;

        match player {
            PlayerColour::WHITE => {
                left_attacked = (position.wp & !A_FILE) << 7;
                right_attacked = (position.wp & !H_FILE) << 9;

                king_check_board |= (enemy_king_pos >> 7) & PositionHelper::bool_to_bitboard(left_attacked & enemy_king_pos > 0);
                king_check_board |= (enemy_king_pos >> 9) & PositionHelper::bool_to_bitboard(right_attacked & enemy_king_pos > 0);
            }
            PlayerColour::BLACK => {
                left_attacked = (position.bp & !H_FILE) >> 7;
                right_attacked = (position.bp & !A_FILE) >> 9;

                king_check_board |= (enemy_king_pos << 7) & PositionHelper::bool_to_bitboard(left_attacked & enemy_king_pos > 0);
                king_check_board |= (enemy_king_pos << 9) & PositionHelper::bool_to_bitboard(right_attacked & enemy_king_pos > 0);
            }
        }

        (left_attacked & position.check_ray_mask, right_attacked & position.check_ray_mask, king_check_board)
    }

    #[inline(always)]
    pub fn add_pawn_movement(move_list: &mut GameMoveList, position: &Position, mut squares: u64, source_square_offset: i8, is_capture: bool) -> u64 {
        let mut valid_movements = 0u64;
        while squares > 0 {
            // trailing_zeros() gives square index from 0..63
            let target_square = squares.trailing_zeros() as u8;
            let source_square = (target_square as i8 + source_square_offset) as u8;
            if position.pin_ray_masks[source_square as usize] & SINGLE_BITBOARDS[target_square as usize] <= 0 {
                squares &= squares - 1;
                continue;
            }
            valid_movements |= SINGLE_BITBOARDS[target_square as usize];

            let target_rank = target_square >> 3;
            if target_rank == 0 || target_rank == 7 {
                move_list.add_move(PieceType::PAWN, source_square, target_square, is_capture, false, PieceType::KNIGHT);
                move_list.add_move(PieceType::PAWN, source_square, target_square, is_capture, false,PieceType::BISHOP);
                move_list.add_move(PieceType::PAWN, source_square, target_square, is_capture, false,PieceType::ROOK);
                move_list.add_move(PieceType::PAWN, source_square, target_square, is_capture, false,PieceType::QUEEN);
            } else {
                move_list.add_move(PieceType::PAWN, source_square, target_square, is_capture, false,PieceType::NONE);
            }

            squares &= squares - 1;
        }

        valid_movements
    }
}

impl Piece for Pawn {
    fn get_piece_type() -> PieceType { PieceType::PAWN }

    #[inline(always)]
    fn calc_attacked_squares(position: &Position, mut _piece_pos: u64, player: &PlayerColour, enemy_king_pos: u64, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        let (left_attacked, right_attacked, king_check_board) = Pawn::calc_left_right_attacks(position, player, enemy_king_pos);

        king_attack_analyzer.merge_check_ray(king_check_board);

        left_attacked | right_attacked
    }

    #[inline(always)]
    fn calc_movements(position: &Position, _piece_pos: u64, move_list: &mut GameMoveList, _enemy_attacked_squares: Option<u64>, _king_attack_analyzer: &mut KingAttackRayAnalyzer) -> (u64, u64) {
        let mut all_valid_movements = 0u64;
        let attacked_squares;

        if position.white_to_move {
            // Calculations for white
            let (left_attacked, right_attacked, _king_attacks) = Pawn::calc_left_right_attacks(position, &PlayerColour::WHITE, position.bk);
            attacked_squares = left_attacked | right_attacked;

            let possible_capture_squares = position.black_occupancy | position.en_passant_sq;
            let capture_squares = attacked_squares & possible_capture_squares;

            all_valid_movements |= Pawn::add_pawn_movement(move_list, &position, left_attacked & capture_squares, -7, true);
            all_valid_movements |= Pawn::add_pawn_movement(move_list, &position, right_attacked & capture_squares, -9, true);

            let forward_one_square_moves = (position.wp << 8) & position.non_occupancy & position.check_ray_mask;
            let forward_two_square_moves = ((position.wp & RANK_2) << 16) & (forward_one_square_moves << 8) & position.non_occupancy & position.check_ray_mask;

            all_valid_movements |= Pawn::add_pawn_movement(move_list, &position, forward_one_square_moves, -8, false);
            all_valid_movements |= Pawn::add_pawn_movement(move_list, &position, forward_two_square_moves, -16, false);

        } else {
            // Calculations for black
            let (left_attacked, right_attacked, _king_attacks) = Pawn::calc_left_right_attacks(position, &PlayerColour::BLACK, position.wk);
            attacked_squares = left_attacked | right_attacked;

            let possible_capture_squares = position.white_occupancy | position.en_passant_sq;
            let capture_squares = attacked_squares & possible_capture_squares;

            all_valid_movements |= Pawn::add_pawn_movement(move_list, &position, left_attacked & capture_squares, 7, true);
            all_valid_movements |= Pawn::add_pawn_movement(move_list, &position, right_attacked & capture_squares, 9, true);

            let forward_one_square_moves = (position.bp >> 8) & position.non_occupancy & position.check_ray_mask;
            let forward_two_square_moves = ((position.bp & RANK_7) >> 16) & (forward_one_square_moves >> 8) & position.non_occupancy & position.check_ray_mask;

            all_valid_movements |= Pawn::add_pawn_movement(move_list, &position, forward_one_square_moves, 8, false);
            all_valid_movements |= Pawn::add_pawn_movement(move_list, &position, forward_two_square_moves, 16, false);
        }

        (attacked_squares, all_valid_movements)
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use crate::test::legalmoveshelper::LegalMovesHelper;

    use super::*;

    #[test]
    fn test_calc_pawn_movements() {
        // 1. Starting position
        let (_, mut position, mut move_list, mut king_attack_analyzer) = LegalMovesHelper::init_test_position_from_fen_str(None);
        LegalMovesHelper::check_attack_and_movement_squares(
            Pawn::calc_movements(&position, position.wp, &mut move_list, None, &mut king_attack_analyzer),
            vec!["a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3"],
            vec!["a3", "a4", "b3", "b4", "c3", "c4", "d3", "d4", "e3", "e4", "f3", "f4", "g3", "g4", "h3", "h4"]
        );

        LegalMovesHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesHelper::check_attack_and_movement_squares(
            Pawn::calc_movements(&position, position.bp, &mut move_list, None, &mut king_attack_analyzer),
            vec!["a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6"],
            vec!["a6", "a5", "b6", "b5", "c6", "c5", "d6", "d5", "e6", "e5", "f6", "f5", "g6", "g5", "h6", "h5"]
        );


        // 2. Typical position with no en-passant or pins
        let (_, mut position, mut move_list, _) = LegalMovesHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/51b1/Q4N1P/P1P2PP1/3RKB1R w KQkq - 1 2"));
        LegalMovesHelper::check_attack_and_movement_squares(
            Pawn::calc_movements(&position,position.wp, &mut move_list, None, &mut king_attack_analyzer),
            vec!["b3", "d3", "e3", "f3", "g3", "h3", "g4", "c6", "d6", "e6", "f6"],
            vec!["c3", "c4", "c6", "d6", "e6", "f6", "g3", "g4", "h4"]
        );

        LegalMovesHelper::switch_sides(&mut position, Some(&mut move_list), None);
        LegalMovesHelper::check_attack_and_movement_squares(
            Pawn::calc_movements(&position,position.bp, &mut move_list, None, &mut king_attack_analyzer),
            vec!["a6", "b6", "c6", "d6", "e6", "f6", "g6", "b5", "d5", "f5", "h5", "b4", "d4"],
            vec!["a6", "a5", "b6", "b5", "c4", "d5", "e6", "h6", "h5"]
        );


        // 3. Position including en-passant on b6 square
        let (_, position, mut move_list, _) = LegalMovesHelper::init_test_position_from_fen_str(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2"));
        LegalMovesHelper::check_attack_and_movement_squares(
            Pawn::calc_movements(&position, position.wp, &mut move_list, None, &mut king_attack_analyzer),
            vec!["b6", "c6", "d6", "e6", "f6", "a8", "c8", "e3", "f3", "g3", "h3", "g4"],
            vec!["a6", "b6", "a8", "b8", "c6", "d6", "e6", "f6", "g3", "g4", "h4"]
        );
    }

    #[test]
    fn test_calc_pawn_movements_benchmark() {
        // let iterations = 40000000;   // currently about 8.5s after calculating and storing pawn moves only
        let iterations = 100;

        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        let mut king_attack_analyzer = KingAttackRayAnalyzer::default();
        let mut move_list = GameMoveList::default();
        let before = Instant::now();
        for _ in 0..iterations {
            move_list.clear();
            Pawn::calc_movements(&position, position.wp,&mut move_list, None, &mut king_attack_analyzer);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
    }

    // #[test]
    // fn test_add_pawn_movements() {
    //     let mut move_list = GameMoveList::default();
    //     Pawn::add_pawn_movement(&mut move_list, 0xff0000u64, -8, false);
    //     assert_eq!(
    //         format!("{:?}", &move_list.move_list[0..move_list.list_len]),
    //         "[GameMove { piece: PAWN, source_square: \"a2\", target_square: \"a3\", promotion_piece: NONE, is_capture: false, is_castling: false }, GameMove { piece: PAWN, source_square: \"b2\", target_square: \"b3\", promotion_piece: NONE, is_capture: false, is_castling: false }, GameMove { piece: PAWN, source_square: \"c2\", target_square: \"c3\", promotion_piece: NONE, is_capture: false, is_castling: false }, GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d3\", promotion_piece: NONE, is_capture: false, is_castling: false }, GameMove { piece: PAWN, source_square: \"e2\", target_square: \"e3\", promotion_piece: NONE, is_capture: false, is_castling: false }, GameMove { piece: PAWN, source_square: \"f2\", target_square: \"f3\", promotion_piece: NONE, is_capture: false, is_castling: false }, GameMove { piece: PAWN, source_square: \"g2\", target_square: \"g3\", promotion_piece: NONE, is_capture: false, is_castling: false }, GameMove { piece: PAWN, source_square: \"h2\", target_square: \"h3\", promotion_piece: NONE, is_capture: false, is_castling: false }]"
    //     );
    //
    //
    //     move_list.clear();
    //     Pawn::add_pawn_movement(
    //         &mut move_list,
    //         PositionHelper::bitboard_from_algebraic(vec!["a8", "d8", "a6", "d3", "h7", "g4"]),
    //         -8,
    //         true
    //     );
    //     assert_eq!(
    //         format!("{:?}", &move_list.move_list[0..move_list.list_len]),
    //         "[GameMove { piece: PAWN, source_square: \"d2\", target_square: \"d3\", promotion_piece: NONE, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"g3\", target_square: \"g4\", promotion_piece: NONE, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"a5\", target_square: \"a6\", promotion_piece: NONE, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"h6\", target_square: \"h7\", promotion_piece: NONE, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a8\", promotion_piece: KNIGHT, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a8\", promotion_piece: BISHOP, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a8\", promotion_piece: ROOK, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"a7\", target_square: \"a8\", promotion_piece: QUEEN, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d8\", promotion_piece: KNIGHT, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d8\", promotion_piece: BISHOP, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d8\", promotion_piece: ROOK, is_capture: true, is_castling: false }, GameMove { piece: PAWN, source_square: \"d7\", target_square: \"d8\", promotion_piece: QUEEN, is_capture: true, is_castling: false }]"
    //     );
    // }
}