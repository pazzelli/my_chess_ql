use std::collections::VecDeque;
use crate::constants::*;
use crate::game::positionhelper::*;
use std::fmt::*;
use std::ops::Deref;
use arrayvec::ArrayString;

// #[derive(Clone, Copy, Debug)]
#[derive(Clone, Copy)]
pub struct GameMove {
    pub piece: PieceType,
    pub source_square: u8,
    pub target_square: u8,
    pub promotion_piece: PieceType,
    pub is_capture: bool,
    pub extended_move_san: ArrayString::<16>
}

impl Default for GameMove {
    fn default() -> Self {
        GameMove {
            piece: PieceType::NONE,
            source_square: 0,
            target_square: 0,
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::<16>::new(),
        }
    }
}

impl GameMove {
    fn get_piece_type_letter(piece_type: PieceType) -> char {
        match piece_type {
            PieceType::PAWN => 'P',
            PieceType::KNIGHT => 'N',
            PieceType::BISHOP => 'B',
            PieceType::ROOK => 'R',
            PieceType::QUEEN => 'Q',
            PieceType::KING => 'K',
            _ => '?'
        }
    }

    pub fn set_extended_san_move_string(&mut self) {
        self.extended_move_san.clear();

        if self.piece == PieceType::KING {
            let square_diff = self.source_square as i16 - self.target_square as i16;
            if square_diff == 2 { self.extended_move_san.push_str("O-O-O"); return; }
            if square_diff == -2 { self.extended_move_san.push_str("O-O"); return; }
        }

        self.extended_move_san.push(GameMove::get_piece_type_letter(self.piece));
        // Doing this each character at a time is significantly faster since these functions
        // don't need to allocate their own strings internally and that turns out to be a bottleneck
        self.extended_move_san.push(PositionHelper::algebraic_file_from_index(self.source_square));
        self.extended_move_san.push(PositionHelper::algebraic_rank_from_index(self.source_square));
        if self.is_capture { self.extended_move_san.push('x'); };
        self.extended_move_san.push(PositionHelper::algebraic_file_from_index(self.target_square));
        self.extended_move_san.push(PositionHelper::algebraic_rank_from_index(self.target_square));
        if self.promotion_piece != PieceType::NONE {
            self.extended_move_san.push('=');
            self.extended_move_san.push(GameMove::get_piece_type_letter(self.promotion_piece));
        }
    }

    // Checks if this move is a match to the partial movement input in Standard Algebraic Notation
    // (e.g. "Nxf3" would match "Ng1xf3" or "Ng1xf3+")
    pub fn is_partial_san_match(&mut self, partial_san: &str) -> bool {
        // Set the extended SAN move string just-in-time
        self.set_extended_san_move_string();

        let mut extended_san_iter = self.extended_move_san.chars();

        let mut partial_san_iter = partial_san.chars().peekable();
        // The loop below checks the back half of the string up to and incl. the dest. square
        loop {
            let partial_san_char = partial_san_iter.next_back();
            if partial_san_char.is_none() { return false };

            let partial_san_char = partial_san_char.unwrap();
            match partial_san_char {
                // For castling moves, just match the whole string directly
                'O' => return (self.extended_move_san.len() >= (partial_san.len() - 1)) && partial_san.starts_with(self.extended_move_san.as_str()),

                // Throw out any check / checkmate or annotation markers
                '+' | '#' | '?' | '!' => { continue; },

                // When the first [a-h] is reached, we've finished capturing the
                // back half of the string, which should match the full san string exactly
                'a'..='h' => {
                    if partial_san_char != extended_san_iter.next_back().unwrap() { return false; }
                    break;
                },

                _ => if partial_san_char != extended_san_iter.next_back().unwrap() { return false; }
            }
        }

        // Now check the first char (i.e. the source piece making the move)
        let first_extended_san_char = extended_san_iter.next().unwrap();

        // Use peek() to see if the piece type was specified at the start of the partial SAN string or not
        let first_partial_san_char = partial_san_iter.peek();

        // There is nothing before the dest square (i.e. pawn moves that are not captures)
        // Return true only if this isn't a pawn move, in which case that initial character is optional
        if first_partial_san_char.is_none() { return first_extended_san_char == 'P'; }
        let first_partial_san_char = first_partial_san_char.unwrap();

        match first_partial_san_char {
            // If piece type was specified, ensure it matches and if so, consume the character
            'B'..='R' => { if &first_extended_san_char != first_partial_san_char { return false; }; partial_san_iter.next(); }

            // If no piece type was specified, ensure it is a pawn move
            _ => { if first_extended_san_char != 'P' { return false; } },
        }

        // Finally, we just need to deal with the (optional) source square (i.e. file and/or rank), if one was specified
        let mut is_extended_source_file_consumed = false;
        let mut is_extended_source_rank_consumed = false;
        for c in partial_san_iter {
            match c {
                'a'..='h' => {
                    if extended_san_iter.next().unwrap() != c { return false; };
                    is_extended_source_file_consumed = true;
                }
                '1'..='8' => {
                    if !is_extended_source_file_consumed { extended_san_iter.next(); is_extended_source_file_consumed = true; }
                    if extended_san_iter.next().unwrap() != c { return false; }
                    is_extended_source_rank_consumed = true;
                },
                'x' => {
                    if !is_extended_source_file_consumed { extended_san_iter.next(); is_extended_source_file_consumed = true; }
                    if !is_extended_source_rank_consumed { extended_san_iter.next(); is_extended_source_rank_consumed = true; }
                    if extended_san_iter.next().unwrap_or(' ') != c { return false; }
                }
                _ => {}
            }
        }

        // If we made it here, then everything matched
        true
    }
}

impl Debug for GameMove {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut result = vec![PositionHelper::algebraic_from_index(self.source_square), PositionHelper::algebraic_from_index(self.target_square)];
        if self.promotion_piece != PieceType::NONE {
            result.push(String::from(match self.promotion_piece {
                PieceType::QUEEN => "q",
                PieceType::KNIGHT => "n",
                PieceType::ROOK => "r",
                PieceType::BISHOP => "b",
                _ => ""
            }));
        }
        f.write_str(result.join("").as_str())

        // f.debug_struct("GameMove")
        //     .field("piece", &self.piece)
        //     .field("source_square", &PositionHelper::algebraic_from_index(self.source_square))
        //     .field("target_square", &PositionHelper::algebraic_from_index(self.target_square))
        //     .field("promotion_piece", &self.promotion_piece)
        //     .field("is_capture", &self.is_capture)
        //     .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_move_extended_san_notation() {
        let mut g = GameMove {
            piece: PieceType::KNIGHT,
            source_square: 6,
            target_square: 21,
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        };
        g.set_extended_san_move_string();
        assert_eq!(g.extended_move_san.as_str(), "Ng1f3");

        g = GameMove {
            piece: PieceType::BISHOP,
            source_square: 0,
            target_square: 63,
            promotion_piece: PieceType::NONE,
            is_capture: true,
            extended_move_san: ArrayString::new()
        };
        g.set_extended_san_move_string();
        assert_eq!(g.extended_move_san.as_str(), "Ba1xh8");

        g = GameMove {
            piece: PieceType::PAWN,
            source_square: 53,
            target_square: 62,
            promotion_piece: PieceType::KNIGHT,
            is_capture: true,
            extended_move_san: ArrayString::new()
        };
        g.set_extended_san_move_string();
        assert_eq!(g.extended_move_san.as_str(), "Pf7xg8=N");
    }

    #[test]
    fn test_game_move_partial_san_match() {
        let mut g = GameMove {
            piece: PieceType::KNIGHT,
            source_square: 6,
            target_square: 21,
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        };
        // Ng1f3
        assert_eq!(g.is_partial_san_match("Ng1f3"), true);
        assert_eq!(g.is_partial_san_match("Ng1xf3"), false);
        assert_eq!(g.is_partial_san_match("Nf3"), true);
        assert_eq!(g.is_partial_san_match("Ng1f3"), true);
        assert_eq!(g.is_partial_san_match("Ngf3"), true);
        assert_eq!(g.is_partial_san_match("N1f3"), true);
        assert_eq!(g.is_partial_san_match("f3"), false);
        assert_eq!(g.is_partial_san_match("Nf3+"), true);
        assert_eq!(g.is_partial_san_match("Ngf3#"), true);
        assert_eq!(g.is_partial_san_match("N1f3#"), true);
        assert_eq!(g.is_partial_san_match("Nh3"), false);

        g = GameMove {
            piece: PieceType::BISHOP,
            source_square: 0,
            target_square: 63,
            promotion_piece: PieceType::NONE,
            is_capture: true,
            extended_move_san: ArrayString::new()
        };
        // "Ba1xh8"
        assert_eq!(g.is_partial_san_match("Ba1xh8"), true);
        assert_eq!(g.is_partial_san_match("Ba1h8"), true);
        assert_eq!(g.is_partial_san_match("Baxh8"), true);
        assert_eq!(g.is_partial_san_match("Bah8"), true);
        assert_eq!(g.is_partial_san_match("B1xh8"), true);
        assert_eq!(g.is_partial_san_match("B1xh8#"), true);
        assert_eq!(g.is_partial_san_match("1xh8"), false);

        g = GameMove {
            piece: PieceType::PAWN,
            source_square: 53,
            target_square: 62,
            promotion_piece: PieceType::KNIGHT,
            is_capture: true,
            extended_move_san: ArrayString::new()
        };
        // Pf7xg8=N
        assert_eq!(g.is_partial_san_match("Pf7xg8=N"), true);
        assert_eq!(g.is_partial_san_match("Pf7xg8=B"), false);
        assert_eq!(g.is_partial_san_match("Pf7xg8=Q"), false);
        assert_eq!(g.is_partial_san_match("Pxg8=N"), true);
        assert_eq!(g.is_partial_san_match("Pg8=N"), true);
        assert_eq!(g.is_partial_san_match("Pg8"), false);
        assert_eq!(g.is_partial_san_match("g8"), false);
        assert_eq!(g.is_partial_san_match("g8=N"), true);
        assert_eq!(g.is_partial_san_match("g7=N"), false);

        g = GameMove {
            piece: PieceType::KING,
            source_square: 4,
            target_square: 2,
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        };
        // O-O-O
        assert_eq!(g.is_partial_san_match("O-O-O"), true);
        assert_eq!(g.is_partial_san_match("O-O"), false);
        assert_eq!(g.is_partial_san_match("O-O-O+"), true);
    }
}