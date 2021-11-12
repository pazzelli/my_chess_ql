use crate::uci::engine::game_move::*;
use crate::uci::engine::constants::*;

#[derive(Debug)]
pub struct GameMoveList {
    // This is much faster than using a Vec<GameMove>
    // Also, there is little benefit in using a tuple() instead of a GameMove struct, so will
    // leave it as a struct for now
    pub move_list: [GameMove; 128],
    // pub move_list: [(u8, u8, bool); 128],   // This was only slightly faster - not worth it
    pub list_len: usize,    // num of elements stored in the list
}

impl Default for GameMoveList {
    fn default() -> GameMoveList{
        GameMoveList {
            move_list: [GameMove::default(); 128],
            list_len: 0
        }
    }
}

impl GameMoveList {
    #[inline(always)]
    pub fn clear(&mut self) {
        self.list_len = 0;
    }

    #[inline(always)]
    pub fn add_move(&mut self, piece: Piece, source_square: u8, target_square: u8, is_capture: bool, promotion_piece: Piece) {
        self.move_list[self.list_len] = GameMove {
            piece,
            source_square,
            target_square,
            promotion_piece,
            is_capture,
        };
        self.list_len += 1;
    }
}