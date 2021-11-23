use std::ops;
use crate::constants::SINGLE_BITBOARDS;
use crate::game::pieces::king::King;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct KingAttackRayAnalyzer {
    pub pin_ray_masks: [u64; 64],
    pub check_ray_mask: u64,
    pub num_checking_pieces: u8,
    pub disable_en_passant: bool
}

impl Default for KingAttackRayAnalyzer {
    fn default() -> Self {
        KingAttackRayAnalyzer {
            pin_ray_masks: [u64::MAX; 64],
            check_ray_mask: u64::MAX,
            num_checking_pieces: 0,
            disable_en_passant: false
        }
    }
}

impl KingAttackRayAnalyzer {
    pub fn clear(&mut self) {
        self.pin_ray_masks = [u64::MAX; 64];
        self.check_ray_mask = u64::MAX;
        self.num_checking_pieces = 0;
        self.disable_en_passant = false;
    }

    #[inline(always)]
    pub fn merge_pin_ray(&mut self, sq_index: usize, pin_ray: u64) {
        // If a pin is present (i.e. pin_ray > 0), set it directly.  Otherwise
        // retain what is already present
        let non_zero_pin_ray = PositionHelper::bool_to_bitboard(pin_ray > 0);
        self.pin_ray_masks[sq_index] = (non_zero_pin_ray & pin_ray) + (!non_zero_pin_ray & self.pin_ray_masks[sq_index]);
    }

    #[inline(always)]
    pub fn merge_check_ray(&mut self, check_ray: u64) {
        // If a check is present (i.e. check_ray > 0), set it directly.  Otherwise
        // retain what is already present
        let is_non_zero_check_ray = check_ray > 0;
        let non_zero_check_ray = PositionHelper::bool_to_bitboard(is_non_zero_check_ray);
        self.check_ray_mask = (non_zero_check_ray & check_ray) + (!non_zero_check_ray & self.check_ray_mask);
        self.num_checking_pieces += is_non_zero_check_ray as u8;
    }

    #[inline(always)]
    pub fn analyze_king_attack_ray(&mut self, position: &Position, attack_ray: u64, is_horizontal_ray: bool, enemy_king_pos: u64) {
        // - calculate the attack ray & all occupancy. If it contains only two pieces (the attacker and the enemy king), then the king is in check
        // - if it contains 3 pieces and the 3rd one is in the friendly occupancy, it must be pinned
        // - if en passant is set and it’s a horizontal ray and contains exactly two pawns (any two pawn will do), and one of them is the en passant pawn (can check in either direction on the board - up or down), then the en passant capture square must be removed from the pawn attack squares (this func will return this as a Boolean)
        // - if it contains 4 or more pieces, there is no check and no pin
        // - this function should return a bit board of pinned pieces, the king attack ray minus the king itself if the king is in check (so other piece movements can be restricted to these squares either to capture the checking piece or to block the check), the number of checking pieces (which would always be 0 or 1 for any given piece type), a bool to indicate if the en passant capture is invalid
        let attack_ray_occupancy = attack_ray & position.all_occupancy;
        let piece_count_along_ray = attack_ray_occupancy.count_ones();
        let attack_ray_friendly_occupancy = attack_ray_occupancy & position.friendly_occupancy;
        let pin_ray = attack_ray
            & PositionHelper::bool_to_bitboard(piece_count_along_ray == 3 && (attack_ray_friendly_occupancy.count_ones() == 2))
            & !enemy_king_pos;

        let pinned_piece_sq_ind = (attack_ray_friendly_occupancy & !enemy_king_pos).trailing_zeros() as usize;
        self.merge_pin_ray(pinned_piece_sq_ind & 63, pin_ray);

        let disable_en_passant_capture = (
            attack_ray_occupancy
                & PositionHelper::bool_to_bitboard(piece_count_along_ray == 4)
                & PositionHelper::bool_to_bitboard(is_horizontal_ray)
                & (position.wp | position.bp)
                & (position.en_passant_sq >> 8 | position.en_passant_sq << 8)
        ) > 0;

        self.disable_en_passant |= disable_en_passant_capture;

        let check_ray = attack_ray
            & PositionHelper::bool_to_bitboard(piece_count_along_ray == 2)
            & !enemy_king_pos;

        self.merge_check_ray(check_ray);
    }
}
