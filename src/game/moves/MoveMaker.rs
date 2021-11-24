use crate::constants::*;
use crate::game::position::*;
use crate::game::moves::gamemove::*;
use crate::game::positionhelper::PositionHelper;

pub struct MoveMaker {
    old_p: u64, old_n: u64, old_b: u64, old_r: u64, old_q: u64, // old_k: u64,
    old_en_passant_sq: u64,
    old_castling_rights: u64
}

impl Default for MoveMaker {
    fn default() -> Self {
        Self {
            old_p: 0, old_n: 0, old_b: 0, old_r: 0, old_q: 0, // old_k: 0,
            old_en_passant_sq: 0,
            old_castling_rights: 0
        }
    }
}

impl MoveMaker {
    #[inline(always)]
    fn calc_make_move_common_objects(white_to_move: bool, en_passant_sq: u64, game_move: &GameMove) -> (u64, u64) {
        let movement_board = SINGLE_BITBOARDS[game_move.source_square as usize] | SINGLE_BITBOARDS[game_move.target_square as usize];
        let mut captured_piece_square = SINGLE_BITBOARDS[game_move.target_square as usize];
        let is_en_passant_capture = en_passant_sq == captured_piece_square;

        // This is basically an if statement that shifts the target capture square up one row if white is capturing en passant
        // or down one row if black is capturing en passant, or just using the capture square if it is not an en passant
        captured_piece_square = (PositionHelper::bool_to_bitboard(is_en_passant_capture && white_to_move) & (captured_piece_square >> 8))
            | (PositionHelper::bool_to_bitboard(is_en_passant_capture && !white_to_move) & (captured_piece_square << 8))
            | (PositionHelper::bool_to_bitboard(!is_en_passant_capture) & captured_piece_square);

        (captured_piece_square, movement_board)
    }

    pub fn make_move(&mut self, position: &mut Position, game_move: &GameMove) {
        let is_pawn_move = game_move.piece == PieceType::PAWN;
        let (captured_piece_square, movement_board) = MoveMaker::calc_make_move_common_objects(position.white_to_move, position.en_passant_sq, &game_move);

        if position.white_to_move {
            match game_move.piece {
                PieceType::PAWN => position.wp ^= movement_board,
                PieceType::KNIGHT => position.wn ^= movement_board,
                PieceType::ROOK => position.wr ^= movement_board,
                PieceType::BISHOP => position.wb ^= movement_board,
                PieceType::QUEEN => position.wq ^= movement_board,
                PieceType::KING => position.wk ^= movement_board,
                PieceType::NONE => ()
            }

            self.old_p = position.bp;
            position.bp ^= PositionHelper::bool_to_bitboard((position.bp & captured_piece_square) > 0) & captured_piece_square;
            self.old_n = position.bn;
            position.bn ^= PositionHelper::bool_to_bitboard((position.bn & captured_piece_square) > 0) & captured_piece_square;
            self.old_b = position.bb;
            position.bb ^= PositionHelper::bool_to_bitboard((position.bb & captured_piece_square) > 0) & captured_piece_square;
            self.old_r = position.br;
            position.br ^= PositionHelper::bool_to_bitboard((position.br & captured_piece_square) > 0) & captured_piece_square;
            self.old_q = position.bq;
            position.bq ^= PositionHelper::bool_to_bitboard((position.bq & captured_piece_square) > 0) & captured_piece_square;

            self.old_castling_rights = position.castling_rights;
            // position.castling_rights &= PositionHelper::bool_to_bitboard(game_move.piece == PieceType::KING)

        } else {
            match game_move.piece {
                PieceType::PAWN => position.bp ^= movement_board,
                PieceType::KNIGHT => position.bn ^= movement_board,
                PieceType::ROOK => position.br ^= movement_board,
                PieceType::BISHOP => position.bb ^= movement_board,
                PieceType::QUEEN => position.bq ^= movement_board,
                PieceType::KING => position.bk ^= movement_board,
                PieceType::NONE => ()
            }

            self.old_p = position.wp;
            position.wp ^= PositionHelper::bool_to_bitboard((position.wp & captured_piece_square) > 0) & captured_piece_square;
            self.old_n = position.wn;
            position.wn ^= PositionHelper::bool_to_bitboard((position.wn & captured_piece_square) > 0) & captured_piece_square;
            self.old_b = position.wb;
            position.wb ^= PositionHelper::bool_to_bitboard((position.wb & captured_piece_square) > 0) & captured_piece_square;
            self.old_r = position.wr;
            position.wr ^= PositionHelper::bool_to_bitboard((position.wr & captured_piece_square) > 0) & captured_piece_square;
            self.old_q = position.wq;
            position.wq ^= PositionHelper::bool_to_bitboard((position.wq & captured_piece_square) > 0) & captured_piece_square;

            position.move_number += 1;
        }

        self.old_en_passant_sq = position.en_passant_sq;
        position.en_passant_sq = PositionHelper::bool_to_bitboard(is_pawn_move && (game_move.target_square as i8 - game_move.source_square as i8).abs() == 16)
            & SINGLE_BITBOARDS[((game_move.source_square + game_move.target_square) >> 1) as usize];

        position.fifty_move_count += !(is_pawn_move || game_move.is_capture) as u8;
        position.white_to_move = !position.white_to_move;
        position.update_occupancy();

        // TODO: need to handle: castling (move rook beside king, updating castling rights), promotions (adding new piece to board)
    }

    pub fn unmake_move(&self, position: &mut Position, game_move: &GameMove) {
        position.white_to_move = !position.white_to_move;

        let is_pawn_move = game_move.piece == PieceType::PAWN;
        let (_, movement_board) = MoveMaker::calc_make_move_common_objects(position.white_to_move, self.old_en_passant_sq, &game_move);

        if position.white_to_move {
            match game_move.piece {
                PieceType::PAWN => position.wp ^= movement_board,
                PieceType::KNIGHT => position.wn ^= movement_board,
                PieceType::ROOK => position.wr ^= movement_board,
                PieceType::BISHOP => position.wb ^= movement_board,
                PieceType::QUEEN => position.wq ^= movement_board,
                PieceType::KING => position.wk ^= movement_board,
                PieceType::NONE => ()
            }

            position.bp = self.old_p;
            position.bn = self.old_n;
            position.bb = self.old_b;
            position.br = self.old_r;
            position.bq = self.old_q;

        } else {
            match game_move.piece {
                PieceType::PAWN => position.bp ^= movement_board,
                PieceType::KNIGHT => position.bn ^= movement_board,
                PieceType::ROOK => position.br ^= movement_board,
                PieceType::BISHOP => position.bb ^= movement_board,
                PieceType::QUEEN => position.bq ^= movement_board,
                PieceType::KING => position.bk ^= movement_board,
                PieceType::NONE => ()
            }

            position.wp = self.old_p;
            position.wn = self.old_n;
            position.wb = self.old_b;
            position.wr = self.old_r;
            position.wq = self.old_q;

            position.move_number -= 1;
        }

        position.en_passant_sq = self.old_en_passant_sq;
        position.castling_rights = self.old_castling_rights;

        position.fifty_move_count -= !(is_pawn_move || game_move.is_capture) as u8;
        position.update_occupancy();
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;
    use std::time::Instant;
    use crate::test::legalmoveshelper::LegalMovesTestHelper;
    use crate::test::movemakertesthelper::MoveMakerTestHelper;

    use super::*;

    #[test]
    fn test_make_move() {
        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/R3KB1R w Q - 1 2"));
        // pxf6
       move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 36, // e5
            target_square: 45, // f6
            promotion_piece: PieceType::NONE,
            is_capture: true,
            is_castling: false
        });
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.bn, vec!["g3"]),
            (position.wp, vec!["a2", "c2", "f2", "g2", "d5", "f6"])
        ]);

        // Rc8
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::ROOK,
            source_square: 56, // a8
            target_square: 58, // c8
            promotion_piece: PieceType::NONE,
            is_capture: false,
            is_castling: false
        });
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.br, vec!["c8", "f8"]),
        ]);

        // Qxc5
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::QUEEN,
            source_square: 16, // a3
            target_square: 34, // c5
            promotion_piece: PieceType::NONE,
            is_capture: true,
            is_castling: false
        });
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wq, vec!["c5"]),
            (position.bp, vec!["a7", "b7", "c6", "e7", "f7", "g6", "h7", "h3"])
        ]);
    }

    #[test]
    fn test_en_passant() {
        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/5np1/2pPP1B1/8/Q5np/P1P2PP1/R3KB1R w Q c6 1 2"));
        // pxc6 (en passant capture)
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 35, // d5
            target_square: 42, // c6
            promotion_piece: PieceType::NONE,
            is_capture: true,
            is_castling: false
        });
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.bp, vec!["a7", "b7", "e7", "f7", "g6", "h7", "h3"]),
            (position.wp, vec!["a2", "c2", "f2", "g2", "c6", "e5"]),
            (position.en_passant_sq, vec![])
        ]);

        // a5 (test setting en_passant_sq in position)
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 48, // a7
            target_square: 32, // a5
            promotion_piece: PieceType::NONE,
            is_capture: false,
            is_castling: false
        });
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.bp, vec!["a5", "b7", "e7", "f7", "g6", "h7", "h3"]),
            (position.en_passant_sq, vec!["a6"])
        ]);

        // c4 (test setting en_passant_sq in position)
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 10, // c2
            target_square: 26, // c4
            promotion_piece: PieceType::NONE,
            is_capture: false,
            is_castling: false
        });
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wp, vec!["a2", "c4", "f2", "g2", "c6", "e5"]),
            (position.en_passant_sq, vec!["c3"])
        ]);
    }

    #[test]
    fn test_unmake_move() {
        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/R3KB1R w Q - 1 2"));
        // pxf6
        MoveMakerTestHelper::test_unmake_move(&mut position, &mut move_maker,&GameMove {
            piece: PieceType::PAWN,
            source_square: 36, // e5
            target_square: 45, // f6
            promotion_piece: PieceType::NONE,
            is_capture: true,
            is_castling: false
        });

        // Qxc5
        MoveMakerTestHelper::test_unmake_move(&mut position, &mut move_maker, &GameMove {
            piece: PieceType::QUEEN,
            source_square: 16, // a3
            target_square: 34, // c5
            promotion_piece: PieceType::NONE,
            is_capture: true,
            is_castling: false
        });

        // d6
        MoveMakerTestHelper::test_unmake_move(&mut position, &mut move_maker, &GameMove {
            piece: PieceType::BISHOP,
            source_square: 5, // f1
            target_square: 12, // e2
            promotion_piece: PieceType::NONE,
            is_capture: false,
            is_castling: false
        });

        // o-o-o
        MoveMakerTestHelper::test_unmake_move(&mut position, &mut move_maker, &GameMove {
            piece: PieceType::KING,
            source_square: 4, // e1
            target_square: 2, // c1
            promotion_piece: PieceType::NONE,
            is_capture: false,
            is_castling: true
        });
    }
}