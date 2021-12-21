use unroll::unroll_for_loops;

use crate::game::position::*;
use crate::constants::*;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::positionhelper::PositionHelper;

struct PositionConverter {}

impl PositionConverter {
    // Encodes the position of a single piece on the input planes for the neural net
    // If black is moving, we need to flip the board vertically
    // Based on the AlphaZero paper, it said to rotate the board by 180 deg, meaning we take (63 - sq_ind)
    // but that would reverse the d- and e-files so I'm opting for a vertical flip using boyo + swap_bytes() instead
    fn encode_piece_positions(piece_pos: u64, piece_planes: &mut [u8; NN_PLANE_COUNT_GAME_PIECE_INPUTS << 6], offset: usize, flip_for_black: bool) {
        let rotate_for_black = flip_for_black as u64;
        let mut piece_pos = (rotate_for_black * piece_pos.swap_bytes()) + ((1 - rotate_for_black) * piece_pos);

        // Find the piece's square index on the board and update it into the final array position
        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;
            // let sq_ind: usize = (rotate_for_black * (63 - sq_ind)) + ((1 - rotate_for_black) * sq_ind);

            // One-hot encode the square on the unique plane where this piece is located
            piece_planes[sq_ind + offset] = 1;

            piece_pos &= piece_pos - 1;
        }
    }

    // Encodes the piece planes for the neural network
    // The planes are ordered according to the current player since the network always 'sees' the board
    // from the current player's perspective
    #[unroll_for_loops]
    fn encode_piece_planes_for_nn(position: &Position) -> [u8; NN_PLANE_COUNT_GAME_PIECE_INPUTS << 6] {
        let mut piece_planes = [0u8; NN_PLANE_COUNT_GAME_PIECE_INPUTS << 6];

        // Convert piece positions (bitboards) into NN planes
        // Neural net inputs are relative to the current player's turn and so for black, the board must also be rotated 180 deg
        let piece_bitboards = if position.white_to_move {
            [position.wp, position.wn, position.wb, position.wr, position.wq, position.wk,
                position.bp, position.bn, position.bb, position.br, position.bq, position.bk]

        } else {
            [position.bp, position.bn, position.bb, position.br, position.bq, position.bk,
                position.wp, position.wn, position.wb, position.wr, position.wq, position.wk]
        };

        // Each piece gets its own plane, with a 1 in an occupied space, 0 otherwise
        for i in 0..piece_bitboards.len() {
            PositionConverter::encode_piece_positions(piece_bitboards[i], &mut piece_planes, i<<6, !position.white_to_move);
        }

        piece_planes
    }

    // Encodes an 'auxiliary' value about one of the game properties into the input planes using
    // a low-level pointer and a fast call to write_bytes() to repeat the given value 64 times as
    // its own input plane
    fn encode_aux_info(aux_value: u8, aux_planes: &mut [u8; NN_PLANE_COUNT_AUX_INPUTS << 6], offset: usize) {
        if aux_value == 0 { return; }   // 0 is the default in the array already

        // Ensure there are 64 more positions available to write this repeated value
        assert!(aux_planes.len() >= offset + 64);
        let aux_planes_ptr = aux_planes.as_mut_ptr();
        unsafe {
            // Use a low-level / unsafe mem function to do this quickly
            aux_planes_ptr.offset(offset as isize).write_bytes(aux_value, 64);
        }
    }

    // Encodes the 'auxiliary' game information each into separate input planes.  In particular:
    // 1x colour, 1x total move count, 1x fifty move count, 2x P1 castling, 2x P2 castling
    // P1 = current player, P2 = opposing player
    #[unroll_for_loops]
    fn encode_aux_planes_for_nn(position: &Position) -> [u8; NN_PLANE_COUNT_AUX_INPUTS << 6] {
        let mut aux_planes = [0u8; NN_PLANE_COUNT_AUX_INPUTS << 6];
        let white_to_move_u64 = position.white_to_move as u64;

        // Manually create the castling planes
        let partial_castling_rights_val_p1 = position.castling_rights & ((white_to_move_u64 * RANK_1) + ((1 - white_to_move_u64) * RANK_8));
        let partial_castling_rights_val_p2 = position.castling_rights & (((1 - white_to_move_u64) * RANK_1) + (white_to_move_u64 * RANK_8));

        // Auxiliary planes list
        let aux_values = [
            !position.white_to_move as u8,
            u16::min(position.move_number, u8::MAX as u16) as u8,
            position.fifty_move_count,
            ((partial_castling_rights_val_p1 & G_FILE) > 0) as u8,
            ((partial_castling_rights_val_p1 & C_FILE) > 0) as u8,
            ((partial_castling_rights_val_p2 & G_FILE) > 0) as u8,
            ((partial_castling_rights_val_p2 & C_FILE) > 0) as u8,
        ];

        // Each auxiliary value gets its own plane with the value simply repeated 64 times on that plane
        for i in 0..aux_values.len() {
            PositionConverter::encode_aux_info(aux_values[i] as u8, &mut aux_planes, i<<6);
        }

        aux_planes
    }

    // Encode a single game movement into the output plane set (to be used to mask out invalid values in the output
    // and to encode the target output)
    // Output planes are encoded using a simple source square vs. target square scheme, with underpromotion
    // moves encoded separately onto their own planes
    fn encode_movement(game_move: &GameMove, movement_planes: &mut [u8; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6]) {
        // Default the plane number to the target square of the piece movement
        let mut plane_num = game_move.target_square as usize;

        // For any move that is an underpromotion, need to encode it on a separate set of planes
        if game_move.promotion_piece != PieceType::NONE && game_move.promotion_piece != PieceType::QUEEN {
            let (_, file_src) = PositionHelper::rank_and_file_from_index(game_move.source_square);
            let (_, file_tgt) = PositionHelper::rank_and_file_from_index(game_move.target_square);

            // Compute differences in source and target squares (rank and file) to determine movement direction
            // let rank_diff: i8 = rank_tgt as i8 - rank_src as i8;
            let file_diff: i8 = file_tgt as i8 - file_src as i8;

            // Since this is a pawn promotion move, file_diff must be one of [-1, 0, 1]
            // and the promotion piece will be one of [knight = 1, bishop = 2, rook = 3]
            // These planes are encoded AFTER the initial 64 planes (one for each target square)
            plane_num = 64 + ((3 * (file_diff + 1)) + (game_move.promotion_piece as i8 - 1)) as usize;
        }

        movement_planes[(plane_num << 6) + game_move.source_square as usize] = 1;


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

    // Encodes all possible game moves for a given position into a set of output planes
    // This will be used to mask out invalid output values before re-normalizing to get the final movement probabilities
    fn encode_movement_output_planes_for_nn (possible_moves: &GameMoveList) -> [u8; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6] {
        let mut movement_planes = [0u8; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6];

        for i in 0..possible_moves.list_len {
            let game_move = &possible_moves.move_list[i];
            PositionConverter::encode_movement(&game_move, &mut movement_planes);
        }
        movement_planes
    }

    // Top-level function to convert a game position into input / output planes for the neural network
    pub fn convert_position_for_nn (position: &Position, possible_moves: &GameMoveList) -> ([u8; NN_PLANE_COUNT_GAME_PIECE_INPUTS << 6], [u8; NN_PLANE_COUNT_AUX_INPUTS << 6], [u8; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6]) {
        let piece_planes = PositionConverter::encode_piece_planes_for_nn(&position);
        let aux_planes = PositionConverter::encode_aux_planes_for_nn(&position);

        let movement_output_planes = PositionConverter::encode_movement_output_planes_for_nn(&possible_moves);

        (piece_planes, aux_planes, movement_output_planes)
    }

    // Converts a target move (for supervised learning) into the set of output planes for the neural network
    pub fn convert_target_move_for_nn (target_move: &GameMove) -> [u8; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6] {
        let mut target_output = [0; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6];
        PositionConverter::encode_movement(&target_move, &mut target_output);
        target_output
    }
}

#[cfg(test)]
mod tests {
    use crate::game::analysis::positionanalyzer::PositionAnalyzer;
    use super::*;

    fn print_nn_encoded_input_plane(plane_data: *const u8, offset: isize, label: &str) {
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

    fn print_nn_encoded_input(piece_planes: &[u8; NN_PLANE_COUNT_GAME_PIECE_INPUTS << 6], aux_planes: &[u8; NN_PLANE_COUNT_AUX_INPUTS << 6], white_to_move: bool) {
        // P N B R Q K
        let piece_labels = if white_to_move {
            ["WP", "WN", "WB", "WR", "WQ", "WK", "BP", "BN", "BB", "BR", "BQ", "BK"]
        } else {
            ["BP", "BN", "BB", "BR", "BQ", "BK", "WP", "WN", "WB", "WR", "WQ", "WK"]
        };
        for i in 0..piece_labels.len() {
            print_nn_encoded_input_plane(piece_planes.as_ptr(), (i<<6) as isize, piece_labels[i]);
        }

        // 1x colour, 1x total move count, 1x fifty move count, 2x P1 castling, 2x P2 castling
        let aux_labels = ["Player Colour", "Move Number", "Fifty Move (100-ply) Count", "P1 Castling - Kingside", "P1 Castling - Queenside", "P2 Castling - Kingside", "P2 Castling - Queenside"];
        for i in 0..aux_labels.len() {
            print_nn_encoded_input_plane(aux_planes.as_ptr(), (i << 6) as isize, aux_labels[i]);
        }
    }

    // Can be used to verify the neural net inputs are being generated correctly
    #[test]
    fn test_convert_position_for_nn() {
        let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/6B1/3PP1b1/Q1P2N2/P4PPP/3RKB1R b Kq g3 4 13"), true).unwrap();
        let mut move_list = GameMoveList::default();
        PositionAnalyzer::calc_legal_moves(&mut position, &mut move_list);
        let (piece_planes, aux_planes, _output_planes) = PositionConverter::convert_position_for_nn(&position, &mut move_list);

        // // Uncomment the next line to see the input plane encodings in all their gory detail
        print_nn_encoded_input(&piece_planes, &aux_planes, position.white_to_move);
    }

    // Checks that every source square x target square mapping as well as underpromotion moves
    // have unique encoding locations within the output planes (i.e. there are no collisions)
    // Turns out there are only 1924 possible moves from any source square to any valid target
    // square (including 3 additional moves for piece underpromotions on each target square)
    // The network specifies 4672 outputs though, so only 41.2% of the outputs will ever be set to 1
    #[test]
    fn test_encode_movement_output_planes() {
        let mut movement_planes = [0; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6];
        let mut target_count = 0;

        for sq_ind in 0..64 {
            let mut cardinal_moves = QUEEN_ATTACKS[sq_ind];
            let mut knight_moves = KNIGHT_ATTACKS[sq_ind];

            let mut game_move = GameMove::default();
            game_move.source_square = sq_ind as u8;

            while cardinal_moves > 0 {
                game_move.piece = PieceType::QUEEN;
                game_move.target_square = cardinal_moves.trailing_zeros() as u8;

                PositionConverter::encode_movement(&game_move, &mut movement_planes);

                // Test underpromotions - these should be encoded in separate planes
                if (game_move.target_square >= 56 && game_move.source_square >= 48 && game_move.source_square < 56)
                    || (game_move.target_square <= 7 && game_move.source_square >= 8 && game_move.source_square < 16) {

                    game_move.piece = PieceType::PAWN;
                    for promotion_piece in [PieceType::KNIGHT, PieceType::BISHOP, PieceType::ROOK] {
                        game_move.promotion_piece = promotion_piece;
                        PositionConverter::encode_movement(&game_move, &mut movement_planes);
                        game_move.promotion_piece = PieceType::NONE;

                        target_count += 1;
                    }
                }

                cardinal_moves &= cardinal_moves - 1;
            }

            while knight_moves > 0 {
                game_move.piece = PieceType::KNIGHT;
                game_move.target_square = knight_moves.trailing_zeros() as u8;

                PositionConverter::encode_movement(&game_move, &mut movement_planes);
                knight_moves &= knight_moves - 1;
            }

            target_count += QUEEN_ATTACKS[sq_ind].count_ones() + KNIGHT_ATTACKS[sq_ind].count_ones();

            let mut current_count = 0u32;
            for s in 0..movement_planes.len() {
                current_count += movement_planes[s] as u32;
            }

            // println!("Square {}: {} total moves", sq_ind, target_count);

            // Ensure each unique move for each unique square has a unique place in the output planes
            assert_eq!(
                target_count,
                current_count
            );
        }
    }
}