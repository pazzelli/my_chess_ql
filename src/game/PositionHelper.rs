// use std::time::Instant;
// use rayon::prelude::*;

use crate::constants::SINGLE_BITBOARDS;
use crate::game::position::Position;

pub struct PositionHelper {

}

impl PositionHelper {
    pub fn rank_and_file_from_index(index: u8) -> (u8, u8) {
        (index >> 3, index & 7)
    }

    pub fn index_from_rank_and_file(rank: u8, file: u8) -> u8 {
        (rank << 3) + file
    }

    pub fn index_from_algebraic(sq: &str) -> u8 {
        let bytes = sq.as_bytes();
        // Use ASCII subtraction to convert ranks (1-8) & files (a-h) into a bit index
        ((bytes[1] - 49) << 3) + (bytes[0] - 97)
    }

    pub fn algebraic_from_index(index: u8) -> String {
        let mut sq = String::from(((index & 7) + 97) as u8 as char);
        sq.push(((index >> 3) + 49) as u8 as char);
        sq
    }

    pub fn bitboard_from_algebraic(squares: Vec<&str>) -> u64 {
        if squares.len() <= 0 { return 0; }

        // squares.par_iter()
        squares.iter()
            .map(|sq| {
                (1u64 << PositionHelper::index_from_algebraic(sq)) as u64
            })
            // Reduce all by ORing them together
            // .reduce(||0, |r, s| r | s)
            .reduce(|r, s| r | s)
            .unwrap()
    }

    pub fn algebraic_from_bitboard(bitboard: u64) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let mut mask: u64 = 0x1;

        for bit_index in 0..64 {
            if mask & bitboard > 0 {
                let mut sq = String::from(((bit_index & 7) + 97) as u8 as char);
                sq.push(((bit_index >> 3) + 49) as u8 as char);

                result.push(PositionHelper::algebraic_from_index(bit_index))
            }
            mask <<= 1;
        }

        result
    }

    /***
    Turns a boolean value into a bitboard, where true gives 0xffffffffffffffff and false gives 0x0
     */
    #[inline(always)]
    pub fn bool_to_bitboard(val: bool) -> u64 {
        u64::MAX * (val as u64)
    }

    pub fn print_position(position: &Position) {
        println!("----------");
        for rank in (0..8).rev() {
            print!("-");
            for file in 0..8 {
                let bb = SINGLE_BITBOARDS[(rank << 3) + file];
                if bb & position.bp > 0 { print!("p"); }
                else if bb & position.bn > 0 { print!("n"); }
                else if bb & position.bb > 0 { print!("b"); }
                else if bb & position.br > 0 { print!("r"); }
                else if bb & position.bq > 0 { print!("q"); }
                else if bb & position.bk > 0 { print!("k"); }
                else if bb & position.wp > 0 { print!("P"); }
                else if bb & position.wn > 0 { print!("N"); }
                else if bb & position.wb > 0 { print!("B"); }
                else if bb & position.wr > 0 { print!("R"); }
                else if bb & position.wq > 0 { print!("Q"); }
                else if bb & position.wk > 0 { print!("K"); }
                else { print!(" "); }
            }
            println!("-");
        }
        println!("----------");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[should_panic(expected = "Divide result is zero")]
    fn test_bitboard_from_algebraic() {
        assert_eq!(PositionHelper::bitboard_from_algebraic(vec![]), 0x0000000000000000);
        assert_eq!(PositionHelper::bitboard_from_algebraic(vec!["a1", "h8"]), 0x8000000000000001);

        // let before = Instant::now();
        // for _ in 0..100 {
            assert_eq!(
                PositionHelper::bitboard_from_algebraic(vec!["a1", "b1", "h1", "g3", "c6", "a8", "b8", "h8"]),
                0x8300040000400083
            );
        // }
        // println!("Elapsed time: {:.2?}", before.elapsed());
    }

    #[test]
    // #[should_panic(expected = "Divide result is zero")]
    fn test_algebraic_from_bitboard() {
        // assert_eq!(PositionHelper::algebraic_from_bitboard(0x0000000000000000), vec![]);
        assert_eq!(PositionHelper::algebraic_from_bitboard(0x8000000000000001), vec!["a1", "h8"]);

        // let before = Instant::now();
        // for _ in 0..100 {
        assert_eq!(
            PositionHelper::algebraic_from_bitboard(0x8300040000400083), vec!["a1", "b1", "h1", "g3", "c6", "a8", "b8", "h8"]
        );
        // }
        // println!("Elapsed time: {:.2?}", before.elapsed());
    }
}