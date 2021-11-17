use crate::constants::*;
use crate::game::position::*;
use crate::game::positionhelper::*;
use crate::game::gamemovelist::*;

pub trait Piece {
    fn calc_attacked_squares(position: &Position, piece_pos: u64, player: &PlayerColour) -> u64;

    fn calc_movements(position: &Position, piece_pos: u64, move_list: &mut GameMoveList, enemy_attacked_squares: Option<u64>) -> (u64, u64);
    // fn calc_movements(position: &Position, mut piece_pos: u64, move_list: &mut GameMoveList, _enemy_attacked_squares: Option<u64>) -> (u64, u64) {
    //     let attacked_squares = Piece::calc_attacked_squares(position, piece_pos, if position.white_to_move {&PlayerColour::WHITE} else {&PlayerColour::BLACK});
    //
    //     while piece_pos > 0 {
    //         let sq_ind: usize = piece_pos.trailing_zeros() as usize;
    //
    //         let capture_squares = attacked_squares & position.enemy_occupancy;
    //         let non_capture_squares = attacked_squares & position.non_occupancy;
    //         Bishop::add_bishop_movement(move_list, sq_ind as u8, capture_squares, true);
    //         Bishop::add_bishop_movement(move_list, sq_ind as u8, non_capture_squares, false);
    //
    //         piece_pos &= piece_pos - 1;
    //     }
    //
    //     (attacked_squares, attacked_squares & !position.friendly_occupancy)
    // }

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