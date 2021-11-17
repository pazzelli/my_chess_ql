use std::ops::Deref;
use crate::constants::*;
use crate::game::position::*;
use crate::game::positionhelper::*;
use crate::game::gamemovelist::*;

pub trait Piece {
    // Each piece type must override and provide a value
    fn get_piece_type() -> PieceType;

    // Unique to each piece type
    fn calc_attacked_squares(_position: &Position, _piece_pos: u64, _player: &PlayerColour) -> u64;

    // Default implementation to add movements for the piece on the source_square to each of the target_squares
    // set to 1 in the target_squares bitboard
    fn add_piece_movement(move_list: &mut GameMoveList, source_square: u8, mut target_squares: u64, is_capture: bool) {
        while target_squares > 0 {
            // trailing_zeros() gives square index from 0..63
            move_list.add_move(Self::get_piece_type(), source_square, target_squares.trailing_zeros() as u8, is_capture, PieceType::NONE);
            target_squares &= target_squares - 1;
        }
    }

    // Default implementation that uses attacked squares to determine movement squares
    // Squares occupied by enemy pieces can be moved to (i.e. a capture) but squares with friendly pieces cannot
    fn calc_movements(position: &Position, mut piece_pos: u64, move_list: &mut GameMoveList, _enemy_attacked_squares: Option<u64>) -> (u64, u64) {
        let attacked_squares = Self::calc_attacked_squares(position, piece_pos, if position.white_to_move {&PlayerColour::WHITE} else {&PlayerColour::BLACK});

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;

            let capture_squares = attacked_squares & position.enemy_occupancy;
            let non_capture_squares = attacked_squares & position.non_occupancy;
            Self::add_piece_movement(move_list, sq_ind as u8, capture_squares, true);
            Self::add_piece_movement(move_list, sq_ind as u8, non_capture_squares, false);

            piece_pos &= piece_pos - 1;
        }

        (attacked_squares, attacked_squares & !position.friendly_occupancy)
    }

    // Helper function to calculate attacks along a rank for rooks / queens
    // Blocking pieces to the right have a higher bit number so can use the efficient o ^ (o - 2s) formula
    // but pieces to the left cannot.  For now I've implemented a simple shifting loop to handle this case
    // Since the math to do this didn't seem simple, and reusing the formula above means I'd have to swap the bits
    // within a byte (and this might tie me too tightly to the x86 architecture if I were to use an endian swap instruction, for eg.)
    fn calc_rank_attacks(position: &Position, source_square: usize, file_index: u8, rank_mask: u64) -> u64 {
        let mut piece_pos = SINGLE_BITBOARDS[source_square];
        let occupancy = position.all_occupancy & rank_mask;
        let blocker = occupancy & !piece_pos;   // o - r

        // If blocker is on a square with a higher bit index (i.e. to the right), can use the o ^ (o - 2s) formula
        // described here: https://www.chessprogramming.org/Subtracting_a_Rook_from_a_Blocking_Piece
        let right_attacks = if blocker > piece_pos {
            occupancy ^ ( blocker - piece_pos )
        } else {
            // Otherwise, just set all bits in front of the slider (on the target rank) to 1
            (!piece_pos + 1) & rank_mask - piece_pos
        };

        // For left (negative rank) attacks, shift the slider bit over by one position each time
        // and check if any piece is present.  The formula above will not work in this case since
        // it only works when the blocker piece is on a square with a higher index number than the slider itself
        let mut left_attacks: u64 = 0;
        for _ in 0..file_index {
            piece_pos >>= 1;
            left_attacks |= piece_pos;
            if piece_pos & position.non_occupancy <= 0 { break; }
        }

        left_attacks | right_attacks
    }

    // Calculates movements for a sliding piece along a file, diagonal or antidiagonal
    // These can all use the o ^ (o - 2s) formula, but for pieces south or west of the source piece,
    // the bytes need to be swapped first (this essentially reverses the bit ordering along the file
    // or diagonal ray so that the subtraction produces the expected result)
    fn calc_file_or_diagonal_attacks(position: &Position, source_square: usize, ray_mask: u64) -> u64 {
        // Below is the Hyperbola Quintessence approach (whatever that means)
        // described here: https://www.chessprogramming.org/Hyperbola_Quintessence
        let piece_pos = SINGLE_BITBOARDS[source_square];
        let ray_occupancy = position.all_occupancy & ray_mask;
        let mut forward = ray_occupancy & !piece_pos;   // o - r
        let mut reverse = forward.swap_bytes();     // o' - r'

        // Ensure there is a blocker bit to borrow from (avoids overflow error)
        if forward > piece_pos {
            forward -= piece_pos;   // o - 2r
            forward ^= ray_occupancy;   // o ^ (o - 2r), as expected
        } else {
            // Sets all bits in front of the slider (on the target ray) to 1
            forward = (!piece_pos + 1) - piece_pos;
        }

        // Ensures there is a blocker bit to borrow from (avoids overflow error)
        let piece_pos_rev = piece_pos.swap_bytes();
        if reverse > piece_pos_rev {
            reverse -= piece_pos_rev;   // o' - 2r'
            reverse ^= ray_occupancy.swap_bytes();
        } else {
            // Sets all bits behind the slider (on the target ray) to 1
            reverse = (!piece_pos_rev + 1) - piece_pos_rev;
        }
        reverse = reverse.swap_bytes();

        (forward | reverse) & ray_mask
    }
}