use crate::constants::*;
use crate::game::pieces::piece::*;
use crate::game::gamemove::*;
use crate::game::gamemovelist::*;
use crate::game::pieces::pawn::*;
use crate::game::pieces::knight::*;
use crate::game::pieces::king::*;
use crate::game::pieces::rook::*;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;
// use core_simd::*;

pub struct MoveGenerator {

}

impl MoveGenerator {
    pub fn calc_all_attacked_squares(position: &Position, player: PlayerColour) -> u64 {
        match player {
            // TODO: add remaining pieces
            PlayerColour::WHITE => {
                Pawn::calc_attacked_squares(position, position.wp, PlayerColour::WHITE) |
                    Knight::calc_attacked_squares(position, position.wn, PlayerColour::WHITE) |
                    King::calc_attacked_squares(position, position.wk, PlayerColour::WHITE) |
                    Rook::calc_attacked_squares(position, position.wr, PlayerColour::WHITE)
            }
            PlayerColour::BLACK => {
                Pawn::calc_attacked_squares(position, position.bp, PlayerColour::BLACK) |
                    Knight::calc_attacked_squares(position, position.bn, PlayerColour::BLACK) |
                    King::calc_attacked_squares(position, position.bk, PlayerColour::BLACK) |
                    Rook::calc_attacked_squares(position, position.br, PlayerColour::BLACK)
            }
        }
    }

    pub fn calc_legal_moves(position: &Position, move_list: &mut GameMoveList) {
        // let mut move_list = GameMoveList::default();

        if position.white_to_move {
            let enemy_attacked_squares: u64 = MoveGenerator::calc_all_attacked_squares(position, PlayerColour::BLACK);

            let (_pawn_attacks, _pawn_movements) = Pawn::calc_movements(position, position.wp, move_list, None);
            let (_knight_attacks, _knight_movements) = Knight::calc_movements(position, position.wn, move_list, None);
            let (_king_attacks, _king_movements) = King::calc_movements(position, position.wk, move_list, Some(enemy_attacked_squares));
            let (_rook_attacks, _rook_movements) = Rook::calc_movements(position, position.wr, move_list, None);

        } else {
            let enemy_attacked_squares: u64 = MoveGenerator::calc_all_attacked_squares(position, PlayerColour::WHITE);

            let (_pawn_attacks, _pawn_movements) = Pawn::calc_movements(position, position.bp, move_list, None);
            let (_knight_attacks, _knight_movements) = Knight::calc_movements(position, position.bn, move_list, None);
            let (_king_attacks, _king_movements) = King::calc_movements(position, position.bk, move_list, Some(enemy_attacked_squares));
            let (_rook_attacks, _rook_movements) = Rook::calc_movements(position, position.br, move_list, None);

        }
        // move_list
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use super::*;
    
    #[test]
    fn test_calc_all_attacked_squares() {
        // *** FYI: THIS TEST WILL FAIL UNTIL I IMPLEMENT THE SLIDING PIECE ATTACK RAYS *** //

        // 1. Starting position
        let position = Position::from_fen(None).unwrap();
        let white_attacks = MoveGenerator::calc_all_attacked_squares(&position, PlayerColour::WHITE);
        let black_attacks = MoveGenerator::calc_all_attacked_squares(&position, PlayerColour::BLACK);

        assert_eq!(white_attacks, PositionHelper::bitboard_from_algebraic(vec!["b1", "c1", "d1", "e1", "f1", "g1", "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2", "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3"]));
        assert_eq!(black_attacks, PositionHelper::bitboard_from_algebraic(vec!["b8", "c8", "d8", "e8", "f8", "g8", "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7", "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6"]));
        
        // TODO: add something more exciting than just the starting position
    }

    #[test]
    fn test_calc_legal_moves_benchmark() {
        let iterations = 10000000;   // currently about 8.5s after calculating and storing pawn moves only
        // let iterations = 100;

        let position = Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
        let mut move_list = GameMoveList::default();
        let before = Instant::now();
        for _ in 0..iterations {
            move_list.clear();
            MoveGenerator::calc_legal_moves(&position, &mut move_list);
            // println!("{:?}", MoveGenerator::calc_legal_moves(&position).move_list);
        }
        println!("Elapsed time: {:.2?}", before.elapsed());
        // println!("{:?}", move_list);
    }

    // #[test]
    // // Result - trailing_zeros() is faster by about 33%
    // fn test_bit_scan_speed() {
    //     let before = Instant::now();
    //     for _ in 0..40000000 {
    //         assert_eq!(0xff0000u64.tzcnt() as u8, 16_u8);
    //     }
    //     println!("tzcnt - Elapsed time: {:.2?}", before.elapsed());
    //
    //     let before = Instant::now();
    //     for _ in 0..40000000 {
    //         assert_eq!(0xff0000u64.trailing_zeros() as u8, 16_u8);
    //     }
    //     println!("trailing_zeros - Elapsed time: {:.2?}", before.elapsed());
    // }
}