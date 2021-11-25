// use array2d::Array2D;
use simple_error::{bail, SimpleError};
use crate::constants::*;
use crate::game::positionhelper::PositionHelper;

#[derive(Clone)]
pub struct Position {
    // Bitboards
    pub wp: u64, pub wn: u64, pub  wb: u64, pub  wr: u64, pub  wq: u64, pub  wk: u64,
    pub bp: u64, pub  bn: u64, pub  bb: u64, pub  br: u64, pub  bq: u64, pub  bk: u64,
    pub en_passant_sq: u64, pub castling_rights: u64,
    // Pin rays limit piece movement (but not the squares the piece is attacking)
    // Check ray limits both since a piece cannot attack a square outside
    // the checking ray if its king is in check
    pub pin_ray_masks: [u64; 64], pub check_ray_mask: u64,
    // squares occupied by either side
    pub white_occupancy: u64, pub black_occupancy: u64, pub all_occupancy: u64, pub non_occupancy: u64,
    pub friendly_occupancy: u64, pub enemy_occupancy: u64,

    pub white_to_move: bool,
    pub king_in_check: bool, pub king_in_double_check: bool,
    pub is_stalemate: bool, pub is_checkmate: bool,
    pub fifty_move_count: u8,
    pub move_number: u16,
}

impl Default for Position {
    fn default() -> Position {
        Position {
            wp: 0, wn: 0, wb: 0, wr: 0, wq: 0, wk: 0,
            bp: 0, bn: 0, bb: 0, br: 0, bq: 0, bk: 0,
            pin_ray_masks: [u64::MAX; 64], check_ray_mask: u64::MAX,
            white_occupancy: 0, black_occupancy: 0, all_occupancy: 0, non_occupancy: 0,
            friendly_occupancy: 0, enemy_occupancy: 0,

            en_passant_sq: 0, castling_rights: 0,
            white_to_move: true,
            king_in_check: false, king_in_double_check: false,
            is_stalemate: false, is_checkmate: false,
            fifty_move_count: 0,
            move_number: 0
        }
    }
}

impl Position {
    pub fn from_fen(fen_str: Option<&str>, print_pos: bool) -> Result<Position, SimpleError> {
        // Default to starting board position
        let pos_str: &str = fen_str.unwrap_or(START_POSITION);
        let pos_str_tokens: Vec<&str> = pos_str.split(' ').collect();
        if pos_str_tokens.len() != 6 { bail!("Invalid FEN string {}", pos_str) }

        let mut position = Position::default();

        // Setup bitboards
        let board_setup_chars = pos_str_tokens[0].chars();
        let mut cur_bit: u64 = 0x0100000000000000;
        for board_setup_char in board_setup_chars {
            match board_setup_char {
                'P' => position.wp |= cur_bit,
                'N' => position.wn |= cur_bit,
                'B' => position.wb |= cur_bit,
                'R' => position.wr |= cur_bit,
                'Q' => position.wq |= cur_bit,
                'K' => position.wk |= cur_bit,
                'p' => position.bp |= cur_bit,
                'n' => position.bn |= cur_bit,
                'b' => position.bb |= cur_bit,
                'r' => position.br |= cur_bit,
                'q' => position.bq |= cur_bit,
                'k' => position.bk |= cur_bit,
                '/' => cur_bit = cur_bit.rotate_right(16),
                '1'..='8' => cur_bit = cur_bit.rotate_left(board_setup_char.to_digit(10).unwrap()),

                _ => bail!("Invalid FEN string {}", pos_str)
            }
            if board_setup_char as u32 > '8' as u32 { cur_bit = cur_bit.rotate_left(1); }
        }

        // Set position properties
        position.white_to_move = pos_str_tokens[1].to_lowercase() != "b";
        if pos_str_tokens[3].len() == 2 {
            position.en_passant_sq = PositionHelper::bitboard_from_algebraic(vec![pos_str_tokens[3]]);
        }
        position.fifty_move_count = pos_str_tokens[4].parse().unwrap();
        position.move_number = pos_str_tokens[5].parse().unwrap();

        // Castling rights
        if pos_str_tokens[2] != "-" {
            for castle_char in pos_str_tokens[2].chars() {
                match castle_char {
                    'K' => position.castling_rights |= SINGLE_BITBOARDS[6],
                    'Q' => position.castling_rights |= SINGLE_BITBOARDS[2],
                    'k' => position.castling_rights |= SINGLE_BITBOARDS[62],
                    'q' => position.castling_rights |= SINGLE_BITBOARDS[58],
                    _ => bail!("Invalid castling side {}", pos_str_tokens[2])
                };
            }
        }

        // Set occupancies
        position.update_occupancy();

        if print_pos { PositionHelper::print_position(&position); }

        Ok(position)
    }

    // pub fn calc_occupancy(&self) -> (u64, u64, u64) {
    pub fn update_occupancy(&mut self) {
        // This approach is about 2x slower since the pipeline doesn't have many operations in it
        // let p1 = Simd::from_array([position.wp, position.wn, position.wb, position.wr]);
        // let p2 = Simd::from_array([position.wq, position.wk, position.bp, position.bn]);
        // let p3 = Simd::from_array([position.bb, position.br, position.bq, position.bk]);
        // (p1 | p2 | p3).horizontal_or()

        // let white_occupancy = position.wp | position.wn | position.wb | position.wr | position.wq | position.wk;
        // let black_occupancy = position.bp | position.bn | position.bb | position.br | position.bq | position.bk;
        //
        // (white_occupancy, black_occupancy, white_occupancy | black_occupancy)

        self.white_occupancy = self.wp | self.wn | self.wb | self.wr | self.wq | self.wk;
        self.black_occupancy = self.bp | self.bn | self.bb | self.br | self.bq | self.bk;
        if self.white_to_move {
            self.friendly_occupancy = self.white_occupancy;
            self.enemy_occupancy = self.black_occupancy;
        } else {
            self.friendly_occupancy = self.black_occupancy;
            self.enemy_occupancy = self.white_occupancy;
        }
        self.all_occupancy = self.white_occupancy | self.black_occupancy;
        self.non_occupancy = !self.all_occupancy;
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_from_fen_start_pos() {
        let position = Position::from_fen(None, true).unwrap();
        assert_eq!(position.bp, 0x00ff000000000000);
        assert_eq!(position.bn, 0x4200000000000000);
        assert_eq!(position.bb, 0x2400000000000000);
        assert_eq!(position.br, 0x8100000000000000);
        assert_eq!(position.bq, 0x0800000000000000);
        assert_eq!(position.bk, 0x1000000000000000);
        assert_eq!(position.wp, 0x000000000000ff00);
        assert_eq!(position.wn, 0x0000000000000042);
        assert_eq!(position.wb, 0x0000000000000024);
        assert_eq!(position.wr, 0x0000000000000081);
        assert_eq!(position.wq, 0x0000000000000008);
        assert_eq!(position.wk, 0x0000000000000010);
        
        assert_eq!(position.white_occupancy, position.wp | position.wn | position.wb | position.wr | position.wq | position.wk);
        assert_eq!(position.black_occupancy, position.bp | position.bn | position.bb | position.br | position.bq | position.bk);
        assert_eq!(position.white_occupancy, position.friendly_occupancy);
        assert_eq!(position.black_occupancy, position.enemy_occupancy);

        assert_eq!(position.white_to_move, true);
        assert_eq!(position.castling_rights, PositionHelper::bitboard_from_algebraic(vec!["c1", "g1", "c8", "g8"]));
        assert_eq!(position.en_passant_sq, 0);
        assert_eq!(position.fifty_move_count, 0);
        assert_eq!(position.move_number, 1);
    }

    #[test]
    fn test_from_fen_pos1() {
        let position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/6B1/3PP1b1/Q1P2N2/P4PPP/3RKB1R b Kq g3 4 13"), true).unwrap();
        assert_eq!(position.bp, PositionHelper::bitboard_from_algebraic(vec!["a7", "b7", "e7", "f7", "h7", "c6", "g6"]));
        assert_eq!(position.bn, PositionHelper::bitboard_from_algebraic(vec!["f6"]));
        assert_eq!(position.bb, PositionHelper::bitboard_from_algebraic(vec!["g7", "g4"]));
        assert_eq!(position.br, PositionHelper::bitboard_from_algebraic(vec!["a8", "f8"]));
        assert_eq!(position.bq, PositionHelper::bitboard_from_algebraic(vec!["d8"]));
        assert_eq!(position.bk, PositionHelper::bitboard_from_algebraic(vec!["g8"]));
        assert_eq!(position.wp, PositionHelper::bitboard_from_algebraic(vec!["d4", "e4", "c3", "a2", "f2", "g2", "h2"]));
        assert_eq!(position.wn, PositionHelper::bitboard_from_algebraic(vec!["f3"]));
        assert_eq!(position.wb, PositionHelper::bitboard_from_algebraic(vec!["g5", "f1"]));
        assert_eq!(position.wr, PositionHelper::bitboard_from_algebraic(vec!["d1", "h1"]));
        assert_eq!(position.wq, PositionHelper::bitboard_from_algebraic(vec!["a3"]));
        assert_eq!(position.wk, PositionHelper::bitboard_from_algebraic(vec!["e1"]));

        assert_eq!(position.white_to_move, false);
        assert_eq!(position.castling_rights, PositionHelper::bitboard_from_algebraic(vec!["g1", "c8"]));
        assert_eq!(position.en_passant_sq, PositionHelper::bitboard_from_algebraic(vec!["g3"]));
        assert_eq!(position.fifty_move_count, 4);
        assert_eq!(position.move_number, 13);
    }

    #[test]
    // #[should_panic(expected = "Divide result is zero")]
    fn test_calc_occupancy() {
        let position = Position::from_fen(Some(START_POSITION), true).unwrap();
        // let before = Instant::now();
        // for _ in 0..100000000 {
        assert_eq!((position.white_occupancy, position.black_occupancy, position.all_occupancy, position.non_occupancy),
                   (0xffff, 0xffff000000000000, 0xffff00000000ffff, 0x0000ffffffff0000));
        // }
        // println!("Elapsed time: {:.2?}", before.elapsed());
    }


}