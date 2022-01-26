use unroll::unroll_for_loops;
use itertools::Itertools;

use crate::game::position::*;
use crate::constants::*;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::pieces::piece::Piece;
use crate::game::positionhelper::PositionHelper;

pub struct NNPositionConverter {
    pub move_history_buffer_white: Vec<f32>,
    pub move_history_buffer_black: Vec<f32>,
}

impl NNPositionConverter {
    pub fn new() -> Self {
        NNPositionConverter {
            move_history_buffer_white: vec![0f32; NN_TOTAL_INPUT_SIZE_PER_POS],
            move_history_buffer_black: vec![0f32; NN_TOTAL_INPUT_SIZE_PER_POS]
        }
    }

    pub fn init_new_game(&mut self) {
        self.move_history_buffer_white = vec![0f32; NN_TOTAL_INPUT_SIZE_PER_POS];
        self.move_history_buffer_black = vec![0f32; NN_TOTAL_INPUT_SIZE_PER_POS];
    }

    // Encodes the position of a single piece on the input planes for the neural net
    // If black is moving, we need to flip the board vertically
    // Based on the AlphaZero paper, it said to rotate the board by 180 deg, meaning we take (63 - sq_ind)
    // but that would reverse the d- and e-files so I'm opting for a vertical flip using boyo + swap_bytes() instead
    fn encode_piece_positions(input_piece_planes: *mut f32, piece_pos: u64, offset: isize, flip_for_black: bool) {
        let rotate_for_black = flip_for_black as u64;
        let mut piece_pos = (rotate_for_black * piece_pos.swap_bytes()) + ((1 - rotate_for_black) * piece_pos);
        // let mut piece_pos = piece_pos;

        // Find the piece's square index on the board and update it into the final array position
        while piece_pos > 0 {
            let sq_ind = piece_pos.trailing_zeros() as isize;
            // let sq_ind: usize = (rotate_for_black * (63 - sq_ind)) + ((1 - rotate_for_black) * sq_ind);

            // One-hot encode the square on the unique plane where this piece is located
            unsafe {
                input_piece_planes.offset(sq_ind + offset).write(1.0);
            }

            piece_pos &= piece_pos - 1;
        }
    }

    // Encodes the piece planes for the neural network
    // The planes are ordered according to the current player since the network always 'sees' the board
    // from the current player's perspective
    #[unroll_for_loops]
    fn encode_piece_planes_for_nn(input_piece_planes: *mut f32, position: &Position, flip_for_black: bool) {
        // Convert piece positions (bitboards) into NN planes
        // Neural net inputs are relative to the current player's turn and so for black, the board must also be flipped vertically
        let piece_bitboards = if !flip_for_black {
            [position.wp, position.wn, position.wb, position.wr, position.wq, position.wk,
                position.bp, position.bn, position.bb, position.br, position.bq, position.bk]

        } else {
            [position.bp, position.bn, position.bb, position.br, position.bq, position.bk,
                position.wp, position.wn, position.wb, position.wr, position.wq, position.wk]
        };

        // Zero out the values on the piece planes since only the piece locations will be changed to 1
        let zero_values = [0f32; NN_PIECE_PLANES << 6];
        unsafe {
            input_piece_planes.copy_from_nonoverlapping(zero_values.as_ptr(), zero_values.len());
        }

        // Each piece gets its own plane, with a 1 in an occupied space, 0 otherwise
        for i in 0..piece_bitboards.len() {
            NNPositionConverter::encode_piece_positions(input_piece_planes, piece_bitboards[i], (i<<6) as isize, flip_for_black);
        }
    }

    // Encodes an 'auxiliary' value about one of the game properties into the input planes using
    // a low-level pointer and a fast call to write_bytes() to repeat the given value 64 times as
    // its own input plane
    fn encode_aux_info(input_aux_planes: *mut f32, aux_value: f32, offset: isize) {
        // if aux_value == 0f32 { return; }   // 0 is the default in the array already
        let values = [aux_value; 64];

        unsafe {
            // Use a low-level / unsafe mem function to do this quickly
            input_aux_planes.offset(offset).copy_from_nonoverlapping(values.as_ptr(), 64);
        }
    }

    // Encodes the 'auxiliary' game information each into separate input planes.  In particular:
    // 1x colour, 1x total move count, 1x fifty move count, 2x P1 castling, 2x P2 castling
    // P1 = current player, P2 = opposing player
    #[unroll_for_loops]
    fn encode_aux_planes_for_nn(input_aux_planes: *mut f32, position: &Position) {
        let white_to_move_u64 = position.white_to_move as u64;

        // Manually create the castling planes
        let partial_castling_rights_val_p1 = position.castling_rights & ((white_to_move_u64 * RANK_1) + ((1 - white_to_move_u64) * RANK_8));
        let partial_castling_rights_val_p2 = position.castling_rights & (((1 - white_to_move_u64) * RANK_1) + (white_to_move_u64 * RANK_8));

        // Auxiliary planes list
        let aux_values = [
            !position.white_to_move as u8 as f32,
            u16::min(position.move_number, u8::MAX as u16) as f32 / 255.0,
            position.fifty_move_count as f32 / 255.0,
            ((partial_castling_rights_val_p1 & G_FILE) > 0) as u8 as f32,
            ((partial_castling_rights_val_p1 & C_FILE) > 0) as u8 as f32,
            ((partial_castling_rights_val_p2 & G_FILE) > 0) as u8 as f32,
            ((partial_castling_rights_val_p2 & C_FILE) > 0) as u8 as f32,
        ];

        // Each auxiliary value gets its own plane with the value simply repeated 64 times on that plane
        for i in 0..aux_values.len() {
            NNPositionConverter::encode_aux_info(input_aux_planes, aux_values[i], (i<<6) as isize);
        }
    }

    // Encode a single game movement into a unique location in the output vector (to be used to mask out invalid values in the output
    // and to encode the target output)
    // Output vector contains one square for every valid / unique / possible move, given the piece movement
    // rules of chess.  Pawn underpromotion moves encoded separately into their own slot since promotion
    // moves share the same source/target square combo but can promote to 4 possible piece types
    pub fn encode_movement(movement_planes: *mut f32, game_move: &GameMove, flip_for_black: bool) {
        let flip_for_black = flip_for_black as u8;
        let source_square = ((flip_for_black * VERTICAL_FLIP_INDICES[game_move.source_square as usize]) + ((1 - flip_for_black) * game_move.source_square)) as u16;
        let target_square = ((flip_for_black * VERTICAL_FLIP_INDICES[game_move.target_square as usize]) + ((1 - flip_for_black) * game_move.target_square)) as u16;
        let promotion_piece = ((game_move.promotion_piece as u8) % 4) as u16;    // makes queen promotions wrap around to 0

        // println!("{}", ((promotion_piece << 12) + (source_square << 6) + target_square));
        let offset = MOVEMENTS_TO_NN_OUTPUT_INDICES[&((promotion_piece << 12) + (source_square << 6) + target_square)];
        unsafe {
            movement_planes.offset(offset as isize).write(1.0);
        }


        // // The approach below is more similar to the AlphaZero paper but it seems unnecessarily tedious
        // // and I never quite got this working in the end.  It also has enough output locations to just
        // // use the cartesian product of all possible source square x target square mappings so I opted for that instead

        // // Knight movements have their own 8 special planes after the 8 * 7 cardinal movement planes
        // let is_knight_movement = (game_move.piece == PieceType::KNIGHT) as i8;
        // let knight_movement_stride: usize = (is_knight_movement as usize) * 8 * 7;
        //
        // // This expression generates a number 0-7 for the cardinal direction stride (diff value for each direction)
        // // or for the 8 possible knight movements
        // let movement_direction_stride: usize = (
        //     (((rank_diff > 0) as i8) << 2) |
        //         (((file_diff > 0) as i8) << 1) |
        //         ((is_knight_movement * (rank_diff == 2) as i8) + ((1 - is_knight_movement) * (rank_diff == file_diff) as i8))
        // ) as usize;
        //
        // // The square movement count only contributes to the plane stride value for cardinal direction (i.e. non-knight) moves
        // let squares_moved = ((1 - is_knight_movement) * (std::cmp::max(rank_diff.abs(), file_diff.abs()))) as usize;
        //
        // // Encode the movement into the output planes
        // movement_planes[((knight_movement_stride + movement_direction_stride + squares_moved) << 6) + game_move.source_square as usize] = 1;
    }

    /// Decodes the neural net output vector index back to a source /target square on the board
    /// The promotion piece / target square aren't needed since the source / target squares alone
    /// are uniquely able to identify the GameMove object for the current position
    /// WARNING: the promotion piece type returned as the 3rd tuple value will return NONE even
    /// if this is a pawn promotion to a QUEEN (this is because a queen promotion and
    /// a non-promotion move both encode the promotion_piece value as 0 for the neural net so it
    /// is ambiguous in this function where the complete list of game moves is no longer available)
    pub fn decode_movement(nn_output_index: u16, flip_for_black: bool) -> (u8, u8, PieceType) {
        let flip_for_black = flip_for_black as u8;

        let movement_index = NN_OUTPUT_INDICES_TO_MOVEMENTS[&nn_output_index];
        let promotion_piece = match movement_index >> 12 {
            1 => PieceType::KNIGHT,
            2 => PieceType::BISHOP,
            3 => PieceType::ROOK,
            _ => PieceType::NONE
        };
        let source_square = ((movement_index >> 6) & 63) as u8;
        let target_square = (movement_index & 63) as u8;

        (
            (flip_for_black * VERTICAL_FLIP_INDICES[source_square as usize]) + ((1 - flip_for_black) * source_square),
            (flip_for_black * VERTICAL_FLIP_INDICES[target_square as usize]) + ((1 - flip_for_black) * target_square),
            promotion_piece
        )
    }

    // Encodes all possible game moves for a given position into a set of output planes
    // This will be used to mask out invalid output values before re-normalizing to get the final movement probabilities
    fn encode_movement_output_planes_for_nn (output_move_mask_planes: *mut f32, possible_moves: &GameMoveList, flip_for_black: bool) {
        for i in 0..possible_moves.list_len {
            let game_move = &possible_moves.move_list[i];
            NNPositionConverter::encode_movement(output_move_mask_planes, &game_move, flip_for_black);
        }
    }

    // Top-level function to convert a game position into input / output planes for the neural network
    pub fn convert_position_for_nn (&mut self, position: &Position, possible_moves: &GameMoveList) -> (Vec<f32>, Vec<f32>) {
        // The encoded input / output arrays to return to the NN for training
        let mut input_data = vec![0f32; NN_TOTAL_INPUT_SIZE_PER_POS];
        let mut output_mask_data = vec![0f32; NN_TOTAL_OUTPUT_SIZE_PER_POS];
        let aux_planes_offset = ((NN_MOVE_HISTORY_PER_POS * NN_PIECE_PLANES) << 6) as isize;

        // Need to write the current encoded position to both the white and black move history buffers
        // but with the position flipped for black (and also the black planes must come first)
        let mut flip_for_black = false;
        for input_piece_planes in [self.move_history_buffer_white.as_mut_ptr(), self.move_history_buffer_black.as_mut_ptr()] {
            unsafe {
                // let input_piece_planes = self.move_history_buffer_white.as_mut_ptr();
                // let input_piece_planes = input_piece_plane_buffer.as_mut_ptr();
                let input_aux_planes = input_piece_planes.offset(aux_planes_offset);

                // Copy the last (N-1) historical positions from the history buffer to the end of the buffer
                // (but before the auxiliary planes)
                input_piece_planes.copy_to(
                    input_piece_planes.offset((NN_PIECE_PLANES << 6) as isize),
                    ((NN_MOVE_HISTORY_PER_POS - 1) * NN_PIECE_PLANES) << 6,
                );

                // Convert the input piece data and auxiliary data into the start of the history buffer
                NNPositionConverter::encode_piece_planes_for_nn(input_piece_planes, &position, flip_for_black);
                NNPositionConverter::encode_aux_planes_for_nn(input_aux_planes, &position);
                flip_for_black = true;
            }
        }

        unsafe {
            // Copy the final contents of the move_history_buffer to the final input array
            // Select the white or black buffer as appropriate
            input_data.as_mut_ptr().copy_from_nonoverlapping(
                if position.white_to_move { self.move_history_buffer_white.as_ptr() } else { self.move_history_buffer_black.as_ptr() },
                NN_TOTAL_INPUT_SIZE_PER_POS,
            );

            // Encode the position outputs directly into the target array, flipping for black if needed
            let output_move_mask_planes = output_mask_data.as_mut_ptr();
            // Create the output movement mask
            NNPositionConverter::encode_movement_output_planes_for_nn(output_move_mask_planes, &possible_moves, !position.white_to_move);
        }

        (input_data, output_mask_data)
    }

    // Converts a target move (for supervised learning) into the set of output planes for the neural network
    pub fn convert_target_move_for_nn (target_move: &GameMove, position: &Position) -> Vec<f32> {
        let mut target_output = vec![0f32; NN_TOTAL_OUTPUT_SIZE_PER_POS];
        NNPositionConverter::encode_movement(target_output.as_mut_ptr(), &target_move, !position.white_to_move);
        target_output
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayString;
    use float_cmp::{approx_eq, assert_approx_eq};
    use std::collections::HashMap;
    use simple_error::bail;
    use crate::game::analysis::positionanalyzer::PositionAnalyzer;
    use crate::game::moves::movemaker::MoveMaker;
    use crate::PGNReader;
    use super::*;

    fn compare_f32_vectors(vec1: &Vec<f32>, vec2: &Vec<f32>) {
        if vec1.len() != vec2.len() {
            println!("Vector lengths mismatch");
            assert_eq!(vec1, vec2);
        }
        for i in 0..vec1.len() {
            assert_approx_eq!(f32, vec1[i], vec2[i]);
        }
    }

    // Returns a vector of checksums for every 32 positions in the input planes
    // 32 is chosen since it is half of the plane size, and this ensures that flipping the
    // board for black will produce unique checksums vs. the unflipped state (i.e. it will
    // catch flipping errors)
    fn calc_encoded_plane_checksums(plane_data: &Vec<f32>) -> Vec<f32> {
        let mut result: Vec<f32> = vec![];
        for stride in (0..plane_data.len()).step_by(32) {
            let mut total = 0f32;
            for i in 0..32 {
                // Ensure we don't run off the end of the input data
                if plane_data.len() <= stride + i {
                    result.push(total);
                    return result;
                }
                total += plane_data[stride + i];
            }
            result.push(total);
        }
        result
    }

    fn print_nn_encoded_plane(plane_data: *const f32, offset: isize, label: &str) {
        println!("\n{}:", label);
        println!("-----------------");
        for rank in (0..8).rev() {
            print!("-");
            for file in 0..8 {
                unsafe {
                    print!("{}-", plane_data.offset(offset + ((rank << 3) + file)).read());
                }
            }
            println!();
        }
        println!("-----------------");
    }

    // fn print_nn_encoded_input(input_planes: &[f32; TOTAL_INPUT_SIZE_PER_POSITION_WITHOUT_HISTORY], white_to_move: bool) {
    fn print_nn_encoded_input(input_planes: &Vec<f32>, white_to_move: bool) {
        // P N B R Q K
        let labels = if white_to_move {
            vec!("WP", "WN", "WB", "WR", "WQ", "WK", "BP", "BN", "BB", "BR", "BQ", "BK")
        } else {
            vec!("BP", "BN", "BB", "BR", "BQ", "BK", "WP", "WN", "WB", "WR", "WQ", "WK")
        };


        // 1x colour, 1x total move count, 1x fifty move count, 2x P1 castling, 2x P2 castling
        // labels.extend_from_slice(&["Player Colour", "Move Number", "Fifty Move (100-ply) Count", "P1 Castling - Kingside", "P1 Castling - Queenside", "P2 Castling - Kingside", "P2 Castling - Queenside"]);
        let aux_labels = vec!["Player Colour", "Move Number", "Fifty Move (100-ply) Count", "P1 Castling - Kingside", "P1 Castling - Queenside", "P2 Castling - Kingside", "P2 Castling - Queenside"];

        for i in 0..(labels.len() * NN_MOVE_HISTORY_PER_POS) {
            print_nn_encoded_plane(input_planes.as_ptr(), (i<<6) as isize, format!("Position {} - {}", -(i as isize / labels.len() as isize), labels[i % labels.len()]).as_str());
        }

        unsafe {
            let input_aux_planes = input_planes.as_ptr().offset(((NN_MOVE_HISTORY_PER_POS * NN_PIECE_PLANES) << 6) as isize);
            for i in 0..aux_labels.len() {
                print_nn_encoded_plane(input_aux_planes, (i << 6) as isize, format!("Position {} - {}", (i / labels.len()), aux_labels[i]).as_str());
            }
        }
    }

    fn print_nn_encoded_output(output_planes: &Vec<f32>, white_to_move: bool) {
        println!("\nOutput plane contents (movements):");

        for i in 0..output_planes.len() {
            if output_planes[i] <= 0f32 { continue; }

            let (source_square, target_square, promotion_piece) = NNPositionConverter::decode_movement(i as u16, !white_to_move);
            let mut game_move = GameMove {
                piece: PieceType::NONE,
                source_square,
                target_square,
                promotion_piece,
                is_capture: false,
                extended_move_san: Default::default()
            };
            game_move.set_extended_san_move_string();

            println!("{}", game_move.extended_move_san);
        }
    }

    // Can be used to verify the neural net inputs are being generated correctly
    #[test]
    fn test_convert_position_for_nn() {
        let mut position = Position::from_fen(Some("r2q1rk1/p4pbp/2p2np1/6B1/3PP1b1/Q1P2N2/Pp2pPPP/3RKB1R b Kq g3 4 13"), true).unwrap();
        let mut move_list = GameMoveList::default();
        PositionAnalyzer::calc_legal_moves(&mut position, &mut move_list);

        let mut nn_converter = NNPositionConverter::new();
        let (input_planes, output_move_mask_planes) = nn_converter.convert_position_for_nn(&position, &mut move_list);

        // // Uncomment the next lines to see the input + output plane encodings in all their gory detail
        // print_nn_encoded_input(&input_planes, position.white_to_move);
        // print_nn_encoded_output(&output_move_mask_planes, position.white_to_move);

        compare_f32_vectors(
            &calc_encoded_plane_checksums(&input_planes),
            &vec![5.0, 2.0, 1.0, 0.0, 1.0, 1.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 7.0, 0.0, 1.0, 1.0, 1.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 32.0, 32.0, 1.6313733, 1.6313733, 0.5019608, 0.5019608, 0.0, 0.0, 32.0, 32.0, 32.0, 32.0, 0.0, 0.0]
        );

        compare_f32_vectors(
            &calc_encoded_plane_checksums(&output_move_mask_planes),
            &vec![2.0, 0.0, 11.0, 1.0, 1.0, 0.0, 2.0, 0.0, 0.0, 0.0, 0.0, 2.0, 2.0, 0.0, 0.0, 1.0, 0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 6.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 4.0, 0.0, 0.0, 0.0, 8.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
        );
    }

    // Checks that every source square x target square mapping as well as underpromotion moves
    // have unique encoding locations within the output planes (i.e. there are no collisions)
    // Turns out there are only 1924 possible moves from any source square to any valid target
    // square (including 3 additional moves for piece underpromotions on each target square)
    // The network specifies 4672 outputs though, so only 41.2% of the outputs will ever be set to 1
    #[test]
    fn test_encode_movement_output_planes() {
        let mut flip_for_black = false; // first pass for white, second for black
        for _ in 0..2 {
            let mut movement_planes = [0f32; NN_TOTAL_OUTPUT_SIZE_PER_POS];
            let mut target_count = 0;

            for sq_ind in 0..64 {
                let mut cardinal_moves = QUEEN_ATTACKS[sq_ind];
                let mut knight_moves = KNIGHT_ATTACKS[sq_ind];

                let mut game_move = GameMove::default();
                game_move.source_square = sq_ind as u8;

                while cardinal_moves > 0 {
                    game_move.piece = PieceType::QUEEN;
                    game_move.target_square = cardinal_moves.trailing_zeros() as u8;

                    NNPositionConverter::encode_movement(movement_planes.as_mut_ptr(), &game_move, flip_for_black);

                    // Test underpromotions - these should be encoded into separate indices
                    if (!flip_for_black && game_move.target_square >= 56 && game_move.source_square >= 48 && game_move.source_square < 56)
                        || (flip_for_black && game_move.target_square <= 7 && game_move.source_square >= 8 && game_move.source_square < 16) {

                        game_move.piece = PieceType::PAWN;
                        for promotion_piece in [PieceType::KNIGHT, PieceType::BISHOP, PieceType::ROOK] {
                            game_move.promotion_piece = promotion_piece;
                            NNPositionConverter::encode_movement(movement_planes.as_mut_ptr(), &game_move, flip_for_black);
                            game_move.promotion_piece = PieceType::NONE;

                            target_count += 1;
                        }
                    }

                    cardinal_moves &= cardinal_moves - 1;
                }

                while knight_moves > 0 {
                    game_move.piece = PieceType::KNIGHT;
                    game_move.target_square = knight_moves.trailing_zeros() as u8;

                    NNPositionConverter::encode_movement(movement_planes.as_mut_ptr(), &game_move, flip_for_black);
                    knight_moves &= knight_moves - 1;
                }

                target_count += QUEEN_ATTACKS[sq_ind].count_ones() + KNIGHT_ATTACKS[sq_ind].count_ones();

                let mut current_count = 0u32;
                for s in 0..movement_planes.len() {
                    current_count += movement_planes[s] as u32;
                }

                println!("Square {}: {} total moves", sq_ind, target_count);

                // Ensure each unique move for each unique square has a unique place in the output planes
                assert_eq!(target_count, current_count);
            }

            flip_for_black = !flip_for_black;
        }
    }

    // // Checks that every source square x target square mapping as well as underpromotion moves
    // // have unique encoding locations within the output vector (i.e. there are no collisions)
    // // Turns out there are only 1858 possible moves from any source square to any valid target
    // // square (including 3 additional moves for piece underpromotions on each target square)
    // #[test]
    // fn test_calc_movements_to_nn_output_map() {
    //     let mut target_count = 0u16;
    //     let mut movement_map = HashMap::<u16, u16>::with_capacity(2048);
    //
    //     for source_square in 0..64 {
    //         let mut cardinal_moves = QUEEN_ATTACKS[source_square];
    //         let mut knight_moves = KNIGHT_ATTACKS[source_square];
    //
    //         while cardinal_moves > 0 {
    //             let target_square = cardinal_moves.trailing_zeros() as usize;
    //
    //             movement_map.insert(((source_square << 6) + target_square) as u16, target_count);
    //             target_count += 1;
    //
    //             // Test underpromotions - these should be encoded in separate planes
    //             // Only need to do this for the rank 7 to 8 promotions since the board will also be flipped
    //             // vertically for black, meaning pawns will only be moving north on the board from the NN's perspective
    //             if target_square >= 56 && source_square >= 48 && source_square < 56 {
    //                 // || (game_move.target_square <= 7 && game_move.source_square >= 8 && game_move.source_square < 16) {
    //                 for promotion_piece in [PieceType::KNIGHT, PieceType::BISHOP, PieceType::ROOK] {
    //                     movement_map.insert(((((promotion_piece as u8) as usize) << 12) + (source_square << 6) + target_square) as u16, target_count);
    //
    //                     target_count += 1;
    //                 }
    //             }
    //
    //             cardinal_moves &= cardinal_moves - 1;
    //         }
    //
    //         while knight_moves > 0 {
    //             let target_square = knight_moves.trailing_zeros() as usize;
    //
    //             movement_map.insert(((source_square << 6) + target_square) as u16, target_count);
    //             target_count += 1;
    //
    //             knight_moves &= knight_moves - 1;
    //         }
    //
    //         println!("Square {}: {} total moves", source_square, target_count);
    //     }
    //
    //     print!("\n[");
    //     for key in movement_map.keys().sorted() {
    //     //     // print!("({}, {}), ", key, movement_map[key]);
    //         print!("({}, {}), ", movement_map[key], key);
    //         // print!("{}, ", key);
    //     }
    //     println!("]");
    //     // println!("{:?}", movement_map);
    // }

    fn test_encode_movement_helper(piece: PieceType, source_square: u8, target_square: u8, promotion_piece: PieceType, flip_for_black: bool, expected_index: u16) {
        let mut movement_planes = [0f32; NN_TOTAL_OUTPUT_SIZE_PER_POS];

        NNPositionConverter::encode_movement(
            movement_planes.as_mut_ptr(),
            &GameMove {
                piece,
                source_square,
                target_square,
                is_capture: false,
                promotion_piece,
                extended_move_san: ArrayString::default(),
            },
            flip_for_black
        );
        assert_eq!(movement_planes[MOVEMENTS_TO_NN_OUTPUT_INDICES[&expected_index] as usize], 1f32);
    }

    #[test]
    fn test_encode_movement_output_plane_locations() {
        // Basic moves for white
        test_encode_movement_helper(PieceType::QUEEN, 0, 1, PieceType::NONE, false, 1);
        test_encode_movement_helper(PieceType::QUEEN, 1, 0, PieceType::NONE, false, 64);
        test_encode_movement_helper(PieceType::QUEEN, 37, 39, PieceType::NONE, false, 2407);
        test_encode_movement_helper(PieceType::QUEEN, 63, 36, PieceType::NONE, false, 4068);
        test_encode_movement_helper(PieceType::QUEEN, 63, 62, PieceType::NONE, false, 4094);

        // // Basic moves for black - the source square index should be flipped vertically
        test_encode_movement_helper(PieceType::QUEEN, 0, 1, PieceType::NONE, true, 3641);
        test_encode_movement_helper(PieceType::QUEEN, 1, 0, PieceType::NONE, true, 3704);
        test_encode_movement_helper(PieceType::QUEEN, 37, 39, PieceType::NONE, true, 1887);
        test_encode_movement_helper(PieceType::QUEEN, 63, 36, PieceType::NONE, true, 476);
        test_encode_movement_helper(PieceType::QUEEN, 63, 62, PieceType::NONE, true, 454);

        // Promotions for white - only underpromotions should be encoded on the additional indices
        test_encode_movement_helper(PieceType::PAWN, 48, 56, PieceType::QUEEN, false, 3128);
        // knight -> 1 * 4096
        test_encode_movement_helper(PieceType::PAWN, 48, 56, PieceType::KNIGHT, false, 7224);
        test_encode_movement_helper(PieceType::PAWN, 48, 56, PieceType::BISHOP, false, 11320);
        test_encode_movement_helper(PieceType::PAWN, 48, 56, PieceType::ROOK, false, 15416);
        // knight -> 1 * 4096
        test_encode_movement_helper(PieceType::PAWN, 54, 61, PieceType::KNIGHT, false, 7613);
        // rook -> 3 * 4096
        test_encode_movement_helper(PieceType::PAWN, 54, 63, PieceType::ROOK, false, 15807);


        // Promotions for black - only underpromotions should be placed on the additional output planes
        test_encode_movement_helper(PieceType::PAWN, 8, 0, PieceType::QUEEN, true, 3128);
        test_encode_movement_helper(PieceType::PAWN, 8, 0, PieceType::KNIGHT, true, 7224);
        test_encode_movement_helper(PieceType::PAWN, 8, 0, PieceType::BISHOP, true, 11320);
        test_encode_movement_helper(PieceType::PAWN, 8, 0, PieceType::ROOK, true, 15416);

        // knight -> 1 * 4096
        test_encode_movement_helper(PieceType::PAWN, 10, 1, PieceType::KNIGHT, true, 7353);
        // rook -> 3 * 4096
        test_encode_movement_helper(PieceType::PAWN, 10, 3, PieceType::ROOK, true, 15547);
    }
    
    #[test]
    fn test_movement_history() {
        let file_path = "src/test/resources/TestMoveHistoryPGN.pgn";
        let mut pgn = PGNReader::init_pgn_file(file_path);
        println!("Testing move history from file: {}", file_path);

        // Game 1:  1. e4 c6 2. d4 d5 3. Nc3 dxe4 4. Nxe4 Bf5 5. Ng3 Bg6  0-1
        // Game 2:  1. d4 Nf6 2. c4 g6 3. Nc3 Bg7 4. e4 O-O 5. Nf3 c5  1-0
        let mut expected_input_checksums: HashMap<i32, Vec<f32>> = HashMap::new();
        let mut expected_output_mask_checksums: HashMap<i32, Vec<f32>> = HashMap::new();
        let mut expected_output_target_checksums: HashMap<i32, Vec<f32>> = HashMap::new();

        // Position 0 (start of game 1):
        expected_input_checksums.insert(0, vec![8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1254902, 0.1254902, 0.0, 0.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0]);
        expected_output_mask_checksums.insert(0,   vec![0.0, 2.0, 0.0, 0.0, 0.0, 2.0, 2.0, 2.0, 3.0, 3.0, 2.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        expected_output_target_checksums.insert(0, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);


        // Position 9 (last move by black before game end - Bg6)
        expected_input_checksums.insert(9, vec![7.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 7.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 7.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 7.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 7.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 7.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 7.0, 1.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 7.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 32.0, 32.0, 0.627451, 0.627451, 0.2509804, 0.2509804, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0]);
        expected_output_mask_checksums.insert(9,   vec![0.0, 2.0, 8.0, 1.0, 0.0, 2.0, 2.0, 2.0, 0.0, 2.0, 1.0, 2.0, 2.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 9.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        expected_output_target_checksums.insert(9, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Position 10 - start of game 2
        expected_input_checksums.insert(10, vec![8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1254902, 0.1254902, 0.0, 0.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0, 32.0]);
        expected_output_mask_checksums.insert(10,   vec![0.0, 2.0, 0.0, 0.0, 0.0, 2.0, 2.0, 2.0, 3.0, 3.0, 2.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        expected_output_target_checksums.insert(10, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);

        // Position 19 - (last move by black before game end - c5)
        expected_input_checksums.insert(19, vec![8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 8.0, 0.0, 2.0, 0.0, 2.0, 0.0, 2.0, 0.0, 1.0, 0.0, 1.0, 32.0, 32.0, 0.627451, 0.627451, 0.2509804, 0.2509804, 0.0, 0.0, 0.0, 0.0, 32.0, 32.0, 32.0, 32.0]);
        expected_output_mask_checksums.insert(19,   vec![0.0, 2.0, 1.0, 1.0, 1.0, 0.0, 2.0, 2.0, 3.0, 3.0, 0.0, 2.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 5.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        expected_output_target_checksums.insert(19, vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);


        let mut pos_count = 0;
        loop {
            let pos_result = pgn.load_next_position();
            if pos_result.is_none() { break; }

            let (input_data, output_mask, output_target, _game_result, _white_to_move, _is_new_game) = pos_result.unwrap();

            // println!("{}", game_result);

            match expected_input_checksums.get(&pos_count) {
                Some(vec) => {
                    // if pos_count == 19 { print_nn_encoded_input(&input_data, white_to_move) };
                    compare_f32_vectors(
                        &calc_encoded_plane_checksums(&input_data),
                        vec,
                    )
                },
                _ => ()
            }

            match expected_output_mask_checksums.get(&pos_count) {
                Some(vec) => {
                    // if pos_count == 19 { print_nn_encoded_output(&output_mask, white_to_move); }
                    compare_f32_vectors(
                        &calc_encoded_plane_checksums(&output_mask),
                        vec,
                    )
                },
                _ => ()
            }

            match expected_output_target_checksums.get(&pos_count) {
                Some(vec) => {
                    // if pos_count == 9 { print_nn_encoded_output(&output_target); }
                    compare_f32_vectors(
                        &calc_encoded_plane_checksums(&output_target),
                        vec,
                    )
                },
                _ => ()
            }

            pos_count += 1;
        }
    }
}