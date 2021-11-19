use std::ops::Deref;
use crate::constants::*;
use crate::game::position::*;
use crate::game::positionhelper::*;
use crate::game::positionanalyzer::*;
use crate::game::gamemovelist::*;

pub trait Piece {
    // Each piece type must override and provide a value
    fn get_piece_type() -> PieceType;

    // Unique to each piece type
    fn calc_attacked_squares(_position: &Position, _piece_pos: u64, _player: &PlayerColour, enemy_king_pos: u64) -> (u64, KingAttackRayAnalysis);

    fn analyze_king_attack_ray(position: &Position, attack_piece_pos: u64, attack_ray: u64, is_horizontal_ray: bool, enemy_king_pos: u64) -> KingAttackRayAnalysis {
        // - calculate the attack ray & all occupancy. If it contains only two pieces (the attacker and the enemy king), then the king is in check
        // - if it contains 3 pieces and the 3rd one is in the friendly occupancy, it must be pinned
        // - if en passant is set and it’s a horizontal ray and contains exactly two pawns (any two pawn will do), and one of them is the en passant pawn (can check in either direction on the board - up or down), then the en passant capture square must be removed from the pawn attack squares (this func will return this as a Boolean)
        // - if it contains 4 or more pieces, there is no check and no pin
        // - this function should return a bit board of pinned pieces, the king attack ray minus the king itself if the king is in check (so other piece movements can be restricted to these squares either to capture the checking piece or to block the check), the number of checking pieces (which would always be 0 or 1 for any given piece type), a bool to indicate if the en passant capture is invalid
        let attack_ray_occupancy = attack_ray & position.all_occupancy;
        let piece_count_along_ray = attack_ray_occupancy.count_ones();
        let pin_board = attack_ray_occupancy
            & PositionHelper::bool_to_bitboard(piece_count_along_ray == 3)
            & position.friendly_occupancy
            & !attack_piece_pos
            & !enemy_king_pos;

        let disable_en_passant_capture = (
            attack_ray_occupancy
                & PositionHelper::bool_to_bitboard(piece_count_along_ray == 4)
                & PositionHelper::bool_to_bitboard(is_horizontal_ray)
                & (position.wp | position.bp)
                & (position.en_passant_sq >> 8 | position.en_passant_sq << 8)
        ) > 0;

        let check_ray = attack_ray
            & PositionHelper::bool_to_bitboard(piece_count_along_ray == 2)
            & !enemy_king_pos;

        KingAttackRayAnalysis(pin_board, check_ray, (check_ray > 0) as u8, disable_en_passant_capture)
    }

    // Default implementation to add movements for the piece on the source_square to each of the target_squares
    // set to 1 in the target_squares bitboard
    fn add_piece_movement(move_list: &mut GameMoveList, source_square: u8, mut target_squares: u64, castling_squares: u64, is_capture: bool) {
        while target_squares > 0 {
            // trailing_zeros() gives square index from 0..63
            let target_square_index = target_squares.trailing_zeros();
            move_list.add_move(Self::get_piece_type(), source_square, target_square_index as u8, is_capture, (SINGLE_BITBOARDS[target_square_index as usize] & castling_squares) > 0, PieceType::NONE);
            target_squares &= target_squares - 1;
        }
    }

    // Default implementation that uses attacked squares to determine movement squares
    // Squares occupied by enemy pieces can be moved to (i.e. a capture) but squares with friendly pieces cannot
    fn calc_movements(position: &Position, mut piece_pos: u64, move_list: &mut GameMoveList, _enemy_attacked_squares: Option<u64>) -> (u64, u64) {
        let (attacked_squares, _possible_king_attacks) = Self::calc_attacked_squares(position, piece_pos, if position.white_to_move {&PlayerColour::WHITE} else {&PlayerColour::BLACK}, 0);

        while piece_pos > 0 {
            let sq_ind: usize = piece_pos.trailing_zeros() as usize;

            let capture_squares = attacked_squares & position.enemy_occupancy;
            let non_capture_squares = attacked_squares & position.non_occupancy;
            Self::add_piece_movement(move_list, sq_ind as u8, capture_squares, 0, true);
            Self::add_piece_movement(move_list, sq_ind as u8, non_capture_squares,0,false);

            piece_pos &= piece_pos - 1;
        }

        (attacked_squares, attacked_squares & !position.friendly_occupancy)
    }

    // Helper function to calculate attacks along a rank for rooks / queens
    // Blocking pieces to the right have a higher bit number so can use the efficient o ^ (o - 2s) formula
    // but pieces to the left cannot.  For now I've implemented a simple shifting loop to handle this case
    // Since the math to do this didn't seem simple, and reusing the formula above means I'd have to swap the bits
    // within a byte (and this might tie me too tightly to the x86 architecture if I were to use an endian swap instruction, for eg.)
    fn calc_rank_attacks(position: &Position, source_square: usize, rank_mask: u64, enemy_king_pos: u64) -> u64 {
        let mut piece_pos = SINGLE_BITBOARDS[source_square];
        // The enemy king position is taken out to avoid an issue where the king is checked by a sliding piece
        // If the king is left into the calculation, then it might try to move to the square behind itself, but
        // still along the ray of the attacking (sliding) piece - the king is removed so it doesn't block this attack ray
        let occupancy = position.all_occupancy & rank_mask & !enemy_king_pos;
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
        let mut left_attacks: u64 = 0u64;
        let non_blocker = !blocker;
        piece_pos >>= 1;
        left_attacks |= piece_pos;
        piece_pos &= non_blocker;
        piece_pos >>= 1;
        left_attacks |= piece_pos;
        piece_pos &= non_blocker;
        piece_pos >>= 1;
        left_attacks |= piece_pos;
        piece_pos &= non_blocker;
        piece_pos >>= 1;
        left_attacks |= piece_pos;
        piece_pos &= non_blocker;
        piece_pos >>= 1;
        left_attacks |= piece_pos;
        piece_pos &= non_blocker;
        piece_pos >>= 1;
        left_attacks |= piece_pos;
        piece_pos &= non_blocker;
        left_attacks |= piece_pos >> 1;
        left_attacks &= rank_mask;


        left_attacks  | right_attacks
    }

    // Calculates movements for a sliding piece along a file, diagonal or antidiagonal
    // These can all use the o ^ (o - 2s) formula, but for pieces south or west of the source piece,
    // the bytes need to be swapped first (this essentially reverses the bit ordering along the file
    // or diagonal ray so that the subtraction produces the expected result)
    fn calc_file_or_diagonal_attacks(position: &Position, source_square: usize, ray_mask: u64, enemy_king_pos: u64) -> u64 {
        // Below is the Hyperbola Quintessence approach (whatever that means)
        // described here: https://www.chessprogramming.org/Hyperbola_Quintessence
        let piece_pos = SINGLE_BITBOARDS[source_square];
        // The enemy king position is taken out to avoid an issue where the king is checked by a sliding piece
        // If the king is left into the calculation, then it might try to move to the square behind itself, but
        // still along the ray of the attacking (sliding) piece - the king is removed so it doesn't block this attack ray
        let ray_occupancy = position.all_occupancy & ray_mask & !enemy_king_pos;
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