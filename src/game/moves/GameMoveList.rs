use std::fmt::*;
use regex::Regex;
use crate::constants::*;
use crate::game::moves::gamemove::*;

const MAX_MOVES_PER_POSITION: usize = 256;

pub struct GameMoveList {
    // This is much faster than using a Vec<GameMove>
    // Also, there is little benefit in using a tuple() instead of a GameMove struct, so will
    // leave it as a struct for now
    pub move_list: [GameMove; MAX_MOVES_PER_POSITION],
    // pub move_list: [(u8, u8, bool); 128],   // This was only slightly faster - not worth it

    // pub piece: [PieceType; MAX_MOVES_PER_POSITION],
    // pub source_square: [u8; MAX_MOVES_PER_POSITION],
    // pub target_square: [u8; MAX_MOVES_PER_POSITION],
    // pub promotion_piece: [PieceType; MAX_MOVES_PER_POSITION],
    // pub is_capture: [bool; MAX_MOVES_PER_POSITION],

    pub list_len: usize,    // num of elements stored in the list
}

impl Default for GameMoveList {
    fn default() -> GameMoveList{
        GameMoveList {
            move_list: [GameMove::default(); 256],
            // piece: [PieceType::NONE; MAX_MOVES_PER_POSITION],
            // source_square: [0; MAX_MOVES_PER_POSITION],
            // target_square: [0; MAX_MOVES_PER_POSITION],
            // promotion_piece: [PieceType::NONE; MAX_MOVES_PER_POSITION],
            // is_capture: [false; MAX_MOVES_PER_POSITION],

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
    pub fn add_move(&mut self, piece: PieceType, source_square: u8, target_square: u8, is_capture: bool, promotion_piece: PieceType) {
        self.move_list[self.list_len] = GameMove {
            piece,
            source_square,
            target_square,
            promotion_piece,
            is_capture
        };

        // self.piece[self.list_len] = piece;
        // self.source_square[self.list_len] = source_square;
        // self.target_square[self.list_len] = target_square;
        // self.promotion_piece[self.list_len] = promotion_piece;
        // self.is_capture[self.list_len] = is_capture;

        self.list_len += 1;
    }

    // TODO: update this to not use Regexes anymore (they are way too slow)
    pub fn get_move_by_extended_san(&self, move_extended_san_re: Regex) -> Option<GameMove> {
        if self.list_len <= 0 { return None };

        for i in 0..self.list_len {
            if move_extended_san_re.find(self.move_list[i].get_extended_san_move_string().as_str()).is_some() {
                return Some(self.move_list[i]);
            }
        }
        None
    }
}

impl Debug for GameMoveList {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        // let result = vec![PositionHelper::algebraic_from_index(self.source_square), PositionHelper::algebraic_from_index(self.target_square)].join("");
        // f.write_str(result.as_str())
        let mut moves: Vec<String> = vec![];
        for i in 0..self.list_len {
            moves.push(format!("{:?}", self.move_list[i]));
            // f.write_str(format!("{:?}\n", self.move_list[i]).as_str());
        }
        moves.sort();
        f.write_str(moves.join(" ").as_str())

        // Ok(())


        // f.debug_struct("GameMove")
        //     .field("piece", &self.piece)
        //     .field("source_square", &PositionHelper::algebraic_from_index(self.source_square))
        //     .field("target_square", &PositionHelper::algebraic_from_index(self.target_square))
        //     .field("promotion_piece", &self.promotion_piece)
        //     .field("is_capture", &self.is_capture)
        //     .finish()
    }
}