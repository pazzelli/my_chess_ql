
use crate::game::positionhelper::PositionHelper;

pub const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[repr(u8)]
pub enum PlayerColour {
    WHITE = 0,
    BLACK = 1,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum PieceType {
    PAWN = 0,
    KNIGHT = 1,
    BISHOP = 2,
    ROOK = 3,
    QUEEN = 4,
    KING = 5,
    NONE = 6,
}

#[repr(u8)]
pub enum CastleSide {
    KINGSIDE = 0,
    QUEENSIDE = 1
}

pub const RANK_1: u64 = 0xff;
pub const RANK_2: u64 = 0xff00;
pub const RANK_3: u64 = 0xff0000;
pub const RANK_4: u64 = 0xff000000;
pub const RANK_5: u64 = 0xff00000000;
pub const RANK_6: u64 = 0xff0000000000;
pub const RANK_7: u64 = 0xff000000000000;
pub const RANK_8: u64 = 0xff00000000000000;

pub const RANKS: [u64; 8] = [RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8];

pub const A_FILE: u64 = 0x0101010101010101;
pub const B_FILE: u64 = 0x0202020202020202;
pub const C_FILE: u64 = 0x0404040404040404;
pub const D_FILE: u64 = 0x0808080808080808;
pub const E_FILE: u64 = 0x1010101010101010;
pub const F_FILE: u64 = 0x2020202020202020;
pub const G_FILE: u64 = 0x4040404040404040;
pub const H_FILE: u64 = 0x8080808080808080;

pub const FILES: [u64; 8] = [A_FILE, B_FILE, C_FILE, D_FILE, E_FILE, F_FILE, G_FILE, H_FILE];

pub const DIAGONALS: [u64; 64] = [0x8040201008040200, 0x80402010080400, 0x804020100800, 0x8040201000, 0x80402000, 0x804000, 0x8000, 0x0, 0x4020100804020000, 0x8040201008040001, 0x80402010080002, 0x804020100004, 0x8040200008, 0x80400010, 0x800020, 0x40, 0x2010080402000000, 0x4020100804000100, 0x8040201008000201, 0x80402010000402, 0x804020000804, 0x8040001008, 0x80002010, 0x4020, 0x1008040200000000, 0x2010080400010000, 0x4020100800020100, 0x8040201000040201, 0x80402000080402, 0x804000100804, 0x8000201008, 0x402010, 0x804020000000000, 0x1008040001000000, 0x2010080002010000, 0x4020100004020100, 0x8040200008040201, 0x80400010080402, 0x800020100804, 0x40201008, 0x402000000000000, 0x804000100000000, 0x1008000201000000, 0x2010000402010000, 0x4020000804020100, 0x8040001008040201, 0x80002010080402, 0x4020100804, 0x200000000000000, 0x400010000000000, 0x800020100000000, 0x1000040201000000, 0x2000080402010000, 0x4000100804020100, 0x8000201008040201, 0x402010080402, 0x0, 0x1000000000000, 0x2010000000000, 0x4020100000000, 0x8040201000000, 0x10080402010000, 0x20100804020100, 0x40201008040201];
pub const ANTI_DIAGONALS: [u64; 64] = [0x0, 0x100, 0x10200, 0x1020400, 0x102040800, 0x10204081000, 0x1020408102000, 0x102040810204000, 0x2, 0x10004, 0x1020008, 0x102040010, 0x10204080020, 0x1020408100040, 0x102040810200080, 0x204081020400000, 0x204, 0x1000408, 0x102000810, 0x10204001020, 0x1020408002040, 0x102040810004080, 0x204081020008000, 0x408102040000000, 0x20408, 0x100040810, 0x10200081020, 0x1020400102040, 0x102040800204080, 0x204081000408000, 0x408102000800000, 0x810204000000000, 0x2040810, 0x10004081020, 0x1020008102040, 0x102040010204080, 0x204080020408000, 0x408100040800000, 0x810200080000000, 0x1020400000000000, 0x204081020, 0x1000408102040, 0x102000810204080, 0x204001020408000, 0x408002040800000, 0x810004080000000, 0x1020008000000000, 0x2040000000000000, 0x20408102040, 0x100040810204080, 0x200081020408000, 0x400102040800000, 0x800204080000000, 0x1000408000000000, 0x2000800000000000, 0x4000000000000000, 0x2040810204080, 0x4081020408000, 0x8102040800000, 0x10204080000000, 0x20408000000000, 0x40800000000000, 0x80000000000000, 0x0];
pub const SINGLE_BITBOARDS: [u64; 64] = [0x1, 0x2, 0x4, 0x8, 0x10, 0x20, 0x40, 0x80, 0x100, 0x200, 0x400, 0x800, 0x1000, 0x2000, 0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000, 0x100000, 0x200000, 0x400000, 0x800000, 0x1000000, 0x2000000, 0x4000000, 0x8000000, 0x10000000, 0x20000000, 0x40000000, 0x80000000, 0x100000000, 0x200000000, 0x400000000, 0x800000000, 0x1000000000, 0x2000000000, 0x4000000000, 0x8000000000, 0x10000000000, 0x20000000000, 0x40000000000, 0x80000000000, 0x100000000000, 0x200000000000, 0x400000000000, 0x800000000000, 0x1000000000000, 0x2000000000000, 0x4000000000000, 0x8000000000000, 0x10000000000000, 0x20000000000000, 0x40000000000000, 0x80000000000000, 0x100000000000000, 0x200000000000000, 0x400000000000000, 0x800000000000000, 0x1000000000000000, 0x2000000000000000, 0x4000000000000000, 0x8000000000000000];

pub const KNIGHT_ATTACKS: [u64; 64] = [0x20400, 0x50800, 0xa1100, 0x142200, 0x284400, 0x508800, 0xa01000, 0x402000, 0x2040004, 0x5080008, 0xa110011, 0x14220022, 0x28440044, 0x50880088, 0xa0100010, 0x40200020, 0x204000402, 0x508000805, 0xa1100110a, 0x1422002214, 0x2844004428, 0x5088008850, 0xa0100010a0, 0x4020002040, 0x20400040200, 0x50800080500, 0xa1100110a00, 0x142200221400, 0x284400442800, 0x508800885000, 0xa0100010a000, 0x402000204000, 0x2040004020000, 0x5080008050000, 0xa1100110a0000, 0x14220022140000, 0x28440044280000, 0x50880088500000, 0xa0100010a00000, 0x40200020400000, 0x204000402000000, 0x508000805000000, 0xa1100110a000000, 0x1422002214000000, 0x2844004428000000, 0x5088008850000000, 0xa0100010a0000000, 0x4020002040000000, 0x400040200000000, 0x800080500000000, 0x1100110a00000000, 0x2200221400000000, 0x4400442800000000, 0x8800885000000000, 0x100010a000000000, 0x2000204000000000, 0x4020000000000, 0x8050000000000, 0x110a0000000000, 0x22140000000000, 0x44280000000000, 0x88500000000000, 0x10a00000000000, 0x20400000000000];
pub const BISHOP_ATTACKS: [u64; 64] = [0x8040201008040200, 0x80402010080500, 0x804020110a00, 0x8041221400, 0x182442800, 0x10204885000, 0x102040810a000, 0x102040810204000, 0x4020100804020002, 0x8040201008050005, 0x804020110a000a, 0x804122140014, 0x18244280028, 0x1020488500050, 0x102040810a000a0, 0x204081020400040, 0x2010080402000204, 0x4020100805000508, 0x804020110a000a11, 0x80412214001422, 0x1824428002844, 0x102048850005088, 0x2040810a000a010, 0x408102040004020, 0x1008040200020408, 0x2010080500050810, 0x4020110a000a1120, 0x8041221400142241, 0x182442800284482, 0x204885000508804, 0x40810a000a01008, 0x810204000402010, 0x804020002040810, 0x1008050005081020, 0x20110a000a112040, 0x4122140014224180, 0x8244280028448201, 0x488500050880402, 0x810a000a0100804, 0x1020400040201008, 0x402000204081020, 0x805000508102040, 0x110a000a11204080, 0x2214001422418000, 0x4428002844820100, 0x8850005088040201, 0x10a000a010080402, 0x2040004020100804, 0x200020408102040, 0x500050810204080, 0xa000a1120408000, 0x1400142241800000, 0x2800284482010000, 0x5000508804020100, 0xa000a01008040201, 0x4000402010080402, 0x2040810204080, 0x5081020408000, 0xa112040800000, 0x14224180000000, 0x28448201000000, 0x50880402010000, 0xa0100804020100, 0x40201008040201];
pub const ROOK_ATTACKS: [u64; 64] = [0x1010101010101fe, 0x2020202020202fd, 0x4040404040404fb, 0x8080808080808f7, 0x10101010101010ef, 0x20202020202020df, 0x40404040404040bf, 0x808080808080807f, 0x10101010101fe01, 0x20202020202fd02, 0x40404040404fb04, 0x80808080808f708, 0x101010101010ef10, 0x202020202020df20, 0x404040404040bf40, 0x8080808080807f80, 0x101010101fe0101, 0x202020202fd0202, 0x404040404fb0404, 0x808080808f70808, 0x1010101010ef1010, 0x2020202020df2020, 0x4040404040bf4040, 0x80808080807f8080, 0x1010101fe010101, 0x2020202fd020202, 0x4040404fb040404, 0x8080808f7080808, 0x10101010ef101010, 0x20202020df202020, 0x40404040bf404040, 0x808080807f808080, 0x10101fe01010101, 0x20202fd02020202, 0x40404fb04040404, 0x80808f708080808, 0x101010ef10101010, 0x202020df20202020, 0x404040bf40404040, 0x8080807f80808080, 0x101fe0101010101, 0x202fd0202020202, 0x404fb0404040404, 0x808f70808080808, 0x1010ef1010101010, 0x2020df2020202020, 0x4040bf4040404040, 0x80807f8080808080, 0x1fe010101010101, 0x2fd020202020202, 0x4fb040404040404, 0x8f7080808080808, 0x10ef101010101010, 0x20df202020202020, 0x40bf404040404040, 0x807f808080808080, 0xfe01010101010101, 0xfd02020202020202, 0xfb04040404040404, 0xf708080808080808, 0xef10101010101010, 0xdf20202020202020, 0xbf40404040404040, 0x7f80808080808080];
pub const QUEEN_ATTACKS: [u64; 64] = [0x81412111090503fe, 0x2824222120a07fd, 0x404844424150efb, 0x8080888492a1cf7, 0x10101011925438ef, 0x2020212224a870df, 0x404142444850e0bf, 0x8182848890a0c07f, 0x412111090503fe03, 0x824222120a07fd07, 0x4844424150efb0e, 0x80888492a1cf71c, 0x101011925438ef38, 0x20212224a870df70, 0x4142444850e0bfe0, 0x82848890a0c07fc0, 0x2111090503fe0305, 0x4222120a07fd070a, 0x844424150efb0e15, 0x888492a1cf71c2a, 0x1011925438ef3854, 0x212224a870df70a8, 0x42444850e0bfe050, 0x848890a0c07fc0a0, 0x11090503fe030509, 0x22120a07fd070a12, 0x4424150efb0e1524, 0x88492a1cf71c2a49, 0x11925438ef385492, 0x2224a870df70a824, 0x444850e0bfe05048, 0x8890a0c07fc0a090, 0x90503fe03050911, 0x120a07fd070a1222, 0x24150efb0e152444, 0x492a1cf71c2a4988, 0x925438ef38549211, 0x24a870df70a82422, 0x4850e0bfe0504844, 0x90a0c07fc0a09088, 0x503fe0305091121, 0xa07fd070a122242, 0x150efb0e15244484, 0x2a1cf71c2a498808, 0x5438ef3854921110, 0xa870df70a8242221, 0x50e0bfe050484442, 0xa0c07fc0a0908884, 0x3fe030509112141, 0x7fd070a12224282, 0xefb0e1524448404, 0x1cf71c2a49880808, 0x38ef385492111010, 0x70df70a824222120, 0xe0bfe05048444241, 0xc07fc0a090888482, 0xfe03050911214181, 0xfd070a1222428202, 0xfb0e152444840404, 0xf71c2a4988080808, 0xef38549211101010, 0xdf70a82422212020, 0xbfe0504844424140, 0x7fc0a09088848281];
pub const KING_ATTACKS: [u64; 64] = [0x302, 0x705, 0xe0a, 0x1c14, 0x3828, 0x7050, 0xe0a0, 0xc040, 0x30203, 0x70507, 0xe0a0e, 0x1c141c, 0x382838, 0x705070, 0xe0a0e0, 0xc040c0, 0x3020300, 0x7050700, 0xe0a0e00, 0x1c141c00, 0x38283800, 0x70507000, 0xe0a0e000, 0xc040c000, 0x302030000, 0x705070000, 0xe0a0e0000, 0x1c141c0000, 0x3828380000, 0x7050700000, 0xe0a0e00000, 0xc040c00000, 0x30203000000, 0x70507000000, 0xe0a0e000000, 0x1c141c000000, 0x382838000000, 0x705070000000, 0xe0a0e0000000, 0xc040c0000000, 0x3020300000000, 0x7050700000000, 0xe0a0e00000000, 0x1c141c00000000, 0x38283800000000, 0x70507000000000, 0xe0a0e000000000, 0xc040c000000000, 0x302030000000000, 0x705070000000000, 0xe0a0e0000000000, 0x1c141c0000000000, 0x3828380000000000, 0x7050700000000000, 0xe0a0e00000000000, 0xc040c00000000000, 0x203000000000000, 0x507000000000000, 0xa0e000000000000, 0x141c000000000000, 0x2838000000000000, 0x5070000000000000, 0xa0e0000000000000, 0x40c0000000000000];

// #[test]
// fn generate_knight_moves() {
//     for index in 0..64 {
//         let mut result = 0u64;
//         let (rank, file) = PositionHelper::rank_and_file_from_index(index);
//
//         for (x, y) in [(1i8, 2i8), (2, 1)] {
//             for multx in [-1, 1] {
//                 for multy in [-1, 1] {
//                     let new_rank = (multx * x) + rank as i8;
//                     let new_file = (multy * y) + file as i8;
//                     if new_rank < 0 || new_rank > 7 || new_file < 0 || new_file > 7 { continue; }
//
//                     result |= 1 << PositionHelper::index_from_rank_and_file(new_rank as u8, new_file as u8);
//                 }
//             }
//         }
//
//         // println!("index {}, moves {:?}", PositionHelper::algebraic_from_index(index), PositionHelper::algebraic_from_bitboard(result));
//         println!("knight_moves[{}] = 0x{}", index, result);
//         // print!("0x{:x}, ", result);
//     }
// }

// #[test]
// fn generate_king_moves() {
//     for index in 0..64 {
//         let mut result = 0u64;
//         let (rank, file) = PositionHelper::rank_and_file_from_index(index);
//
//         for x in [-1, 0, 1] {
//             for y in [-1, 0, 1] {
//                 if (x, y) == (0, 0) { continue; }
//                 let new_rank = x + rank as i8;
//                 let new_file = y + file as i8;
//                 if new_rank < 0 || new_rank > 7 || new_file < 0 || new_file > 7 { continue; }
//
//                 result |= 1 << PositionHelper::index_from_rank_and_file(new_rank as u8, new_file as u8);
//             }
//         }
//
//         // println!("index {}, moves {:?}", PositionHelper::algebraic_from_index(index), PositionHelper::algebraic_from_bitboard(result));
//         // println!("king_moves[{}] = 0x{}", index, result);
//         print!("0x{:x}, ", result);
//     }
// }

// #[test]
// fn generate_diagonal_attacks() {
//     let mut diag_results: Vec<u64> = Vec::with_capacity(64);
//     let mut anti_diag_results: Vec<u64> = Vec::with_capacity(64);
//
//     for index in 0..64 {
//         let mut diag_result = 0u64;
//         let mut anti_diag_result = 0u64;
//         let (rank, file) = PositionHelper::rank_and_file_from_index(index);
//
//
//         for x in [1, -1] {
//             for y in [1, -1] {
//                 let mut new_rank: i8 = rank as i8;
//                 let mut new_file: i8 = file as i8;
//
//                 loop {
//                     new_rank += x as i8;
//                     new_file += y as i8;
//                     if new_rank < 0 || new_rank > 7 || new_file < 0 || new_file > 7 { break; }
//
//                     if x == y {
//                         diag_result |= 1 << PositionHelper::index_from_rank_and_file(new_rank as u8, new_file as u8);
//                     } else {
//                         anti_diag_result |= 1 << PositionHelper::index_from_rank_and_file(new_rank as u8, new_file as u8);
//                     }
//                     // result |= 1 << PositionHelper::index_from_rank_and_file(new_rank as u8, new_file as u8);
//                 }
//             }
//         }
//
//         diag_results.push(diag_result);
//         anti_diag_results.push(anti_diag_result);
//
//         // println!("index {}, moves {:?}", PositionHelper::algebraic_from_index(index), PositionHelper::algebraic_from_bitboard(diag_result));
//         // println!("index {}, moves {:?}", PositionHelper::algebraic_from_index(index), PositionHelper::algebraic_from_bitboard(anti_diag_result));
//     }
//     for index in 0..64 {
//         print!("0x{:x}, ", diag_results[index]);
//     }
//     println!();
//     for index in 0..64 {
//         print!("0x{:x}, ", anti_diag_results[index]);
//     }
//
//     // for index in 0..64 {
//     //     assert_eq!(DIAGONALS[index] | ANTI_DIAGONALS[index], BISHOP_ATTACKS[index]);
//     // }
// }

// #[test]
// fn generate_single_bit_masks() {
//     for index in 0..64 {
//         let result: u64 = 1 << index;
//
//         // println!("index {}, moves {:?}", PositionHelper::algebraic_from_index(index), PositionHelper::algebraic_from_bitboard(result));
//         // println!("king_moves[{}] = 0x{}", index, result);
//         print!("0x{:x}, ", result);
//     }
// }

// #[test]
// fn generate_bishop_moves() {
//     for index in 0..64 {
//         let mut result = 0u64;
//         let (rank, file) = PositionHelper::rank_and_file_from_index(index);
//
//         for x in [1, -1] {
//             for y in [1, -1] {
//                 let mut new_rank: i8 = rank as i8;
//                 let mut new_file: i8 = file as i8;
//
//                 loop {
//                     new_rank += x as i8;
//                     new_file += y as i8;
//                     if new_rank < 0 || new_rank > 7 || new_file < 0 || new_file > 7 { break; }
//
//                     result |= 1 << PositionHelper::index_from_rank_and_file(new_rank as u8, new_file as u8);
//                 }
//             }
//         }
//
//         // println!("index {}, moves {:?}", PositionHelper::algebraic_from_index(index), PositionHelper::algebraic_from_bitboard(result));
//         // println!("king_moves[{}] = 0x{}", index, result);
//         print!("0x{:x}, ", result);
//     }
// }

// #[test]
// fn generate_rook_moves() {
//     for index in 0..64 {
//         let mut result = 0u64;
//         let (rank, file) = PositionHelper::rank_and_file_from_index(index);
//
//         for (x,y) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
//             let mut new_rank: i8 = rank as i8;
//             let mut new_file: i8 = file as i8;
//
//             loop {
//                 new_rank += x as i8;
//                 new_file += y as i8;
//                 if new_rank < 0 || new_rank > 7 || new_file < 0 || new_file > 7 { break; }
//
//                 result |= 1 << PositionHelper::index_from_rank_and_file(new_rank as u8, new_file as u8);
//             }
//         }
//
//         // println!("index {}, moves {:?}", PositionHelper::algebraic_from_index(index), PositionHelper::algebraic_from_bitboard(result));
//         // println!("king_moves[{}] = 0x{}", index, result);
//         print!("0x{:x}, ", result);
//     }
// }

// #[test]
// fn generate_queen_moves() {
//     for index in 0..64 {
//         let mut result = BISHOP_MOVES[index] | ROOK_MOVES[index];
//
//         // println!("index {}, moves {:?}", PositionHelper::algebraic_from_index(index as u8), PositionHelper::algebraic_from_bitboard(result));
//         // println!("king_moves[{}] = 0x{}", index, result);
//         print!("0x{:x}, ", result);
//     }
// }