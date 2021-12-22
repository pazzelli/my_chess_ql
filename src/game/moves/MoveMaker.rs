use crate::constants::*;
use crate::game::position::*;
use crate::game::moves::gamemove::*;
use crate::game::positionhelper::PositionHelper;

pub struct MoveMaker {
    old_wp: u64, old_wn: u64, old_wb: u64, old_wr: u64, old_wq: u64, old_wk: u64,
    old_bp: u64, old_bn: u64, old_bb: u64, old_br: u64, old_bq: u64, old_bk: u64,
    old_en_passant_sq: u64,
    old_castling_rights: u64,
    old_fifty_move_count: u8,
    old_king_in_check: bool, old_king_in_double_check: bool,
    old_is_stalemate: bool, old_is_checkmate: bool,
    old_pin_ray_masks: [u64; 64], old_check_ray_mask: u64,
}

impl Default for MoveMaker {
    fn default() -> Self {
        Self {
            old_wp: 0, old_wn: 0, old_wb: 0, old_wr: 0, old_wq: 0, old_wk: 0,
            old_bp: 0, old_bn: 0, old_bb: 0, old_br: 0, old_bq: 0, old_bk: 0,
            old_en_passant_sq: 0,
            old_castling_rights: 0,
            old_fifty_move_count: 0,
            old_king_in_check: false, old_king_in_double_check: false,
            old_is_stalemate: false, old_is_checkmate: false,
            old_pin_ray_masks: [0u64; 64], old_check_ray_mask: 0
        }
    }
}

impl MoveMaker {
    #[inline(always)]
    fn save_position_state(&mut self, position: &Position) {
        self.old_wp = position.wp;
        self.old_wn = position.wn;
        self.old_wb = position.wb;
        self.old_wr = position.wr;
        self.old_wq = position.wq;
        self.old_wk = position.wk;
        self.old_bp = position.bp;
        self.old_bn = position.bn;
        self.old_bb = position.bb;
        self.old_br = position.br;
        self.old_bq = position.bq;
        self.old_bk = position.bk;
        self.old_en_passant_sq = position.en_passant_sq;
        self.old_castling_rights = position.castling_rights;
        self.old_fifty_move_count = position.fifty_move_count;
        self.old_king_in_check = position.king_in_check;
        self.old_king_in_double_check = position.king_in_double_check;
        self.old_is_stalemate = position.is_stalemate;
        self.old_is_checkmate = position.is_checkmate;
        self.old_pin_ray_masks = position.pin_ray_masks.clone();
        self.old_check_ray_mask = position.check_ray_mask;
    }

    #[inline(always)]
    fn calc_make_move_common_objects(white_to_move: bool, is_pawn_move: bool, en_passant_sq: u64, game_move: &GameMove) -> (u64, u64) {
        let movement_board = SINGLE_BITBOARDS[game_move.source_square as usize] | SINGLE_BITBOARDS[game_move.target_square as usize];
        let mut captured_piece_square = SINGLE_BITBOARDS[game_move.target_square as usize];
        let is_en_passant_capture = en_passant_sq == captured_piece_square && is_pawn_move;

        // This is basically an if statement that shifts the target capture square up one row if white is capturing en passant
        // or down one row if black is capturing en passant, or just using the capture square if it is not an en passant
        captured_piece_square = (PositionHelper::bool_to_bitboard(is_en_passant_capture && white_to_move) & (captured_piece_square >> 8))
            | (PositionHelper::bool_to_bitboard(is_en_passant_capture && !white_to_move) & (captured_piece_square << 8))
            | (PositionHelper::bool_to_bitboard(!is_en_passant_capture) & captured_piece_square);

        (captured_piece_square, movement_board)
    }

    pub fn make_move(&mut self, position: &mut Position, game_move: &GameMove, save_existing_state: bool) {
        let is_pawn_move = game_move.piece == PieceType::PAWN;
        let (target_square_board, movement_board) = MoveMaker::calc_make_move_common_objects(position.white_to_move, is_pawn_move, position.en_passant_sq, &game_move);
        let movement_sq_count = (game_move.target_square as i8 - game_move.source_square as i8).abs();
        let center_movement_sq_board = SINGLE_BITBOARDS[((game_move.source_square + game_move.target_square) >> 1) as usize];

        if save_existing_state { self.save_position_state(position); };

        // Any movement either from a corner square or to a corner square removes castling rights at that location
        let remove_castling_rights_board = CORNERS & movement_board;
        position.castling_rights &= !(remove_castling_rights_board >> 1 | remove_castling_rights_board << 2);

        if position.white_to_move {
            match game_move.piece {
                PieceType::PAWN => position.wp ^= movement_board,
                PieceType::KNIGHT => position.wn ^= movement_board,
                PieceType::BISHOP => position.wb ^= movement_board,
                PieceType::QUEEN => position.wq ^= movement_board,
                PieceType::ROOK => position.wr ^= movement_board,
                PieceType::KING => {
                    position.wk ^= movement_board;
                    // Any king move removes castling rights on both sides
                    position.castling_rights &= !RANK_1;
                    // Move rook to be beside the king, if the king has just moved 2 squares (i.e. castled)
                    position.wr ^= PositionHelper::bool_to_bitboard(movement_sq_count == 2)
                        & (center_movement_sq_board | ((target_square_board << 1 | target_square_board >> 2) & CORNERS));
                },
                PieceType::NONE => ()
            }

            if game_move.promotion_piece != PieceType::NONE {
                position.wp ^= target_square_board;
                match game_move.promotion_piece {
                    PieceType::KNIGHT => position.wn ^= target_square_board,
                    PieceType::ROOK => position.wr ^= target_square_board,
                    PieceType::BISHOP => position.wb ^= target_square_board,
                    PieceType::QUEEN => position.wq ^= target_square_board,
                    _ => ()
                }
            }

            position.bp ^= PositionHelper::bool_to_bitboard((position.bp & target_square_board) > 0) & target_square_board;
            position.bn ^= PositionHelper::bool_to_bitboard((position.bn & target_square_board) > 0) & target_square_board;
            position.bb ^= PositionHelper::bool_to_bitboard((position.bb & target_square_board) > 0) & target_square_board;
            position.br ^= PositionHelper::bool_to_bitboard((position.br & target_square_board) > 0) & target_square_board;
            position.bq ^= PositionHelper::bool_to_bitboard((position.bq & target_square_board) > 0) & target_square_board;

        } else {
            match game_move.piece {
                PieceType::PAWN => position.bp ^= movement_board,
                PieceType::KNIGHT => position.bn ^= movement_board,
                PieceType::BISHOP => position.bb ^= movement_board,
                PieceType::QUEEN => position.bq ^= movement_board,
                PieceType::ROOK => position.br ^= movement_board,
                PieceType::KING => {
                    position.bk ^= movement_board;
                    // Any king move removes castling rights on both sides
                    position.castling_rights &= !RANK_8;
                    // Move rook to be beside the king, if the king has just moved 2 squares (i.e. castled)
                    position.br ^= PositionHelper::bool_to_bitboard(movement_sq_count == 2)
                        & (center_movement_sq_board | ((target_square_board << 1 | target_square_board >> 2) & CORNERS));
                },
                PieceType::NONE => ()
            }

            if game_move.promotion_piece != PieceType::NONE {
                position.bp ^= target_square_board;
                match game_move.promotion_piece {
                    PieceType::KNIGHT => position.bn ^= target_square_board,
                    PieceType::ROOK => position.br ^= target_square_board,
                    PieceType::BISHOP => position.bb ^= target_square_board,
                    PieceType::QUEEN => position.bq ^= target_square_board,
                    _ => ()
                }
            }

            position.wp ^= PositionHelper::bool_to_bitboard((position.wp & target_square_board) > 0) & target_square_board;
            position.wn ^= PositionHelper::bool_to_bitboard((position.wn & target_square_board) > 0) & target_square_board;
            position.wb ^= PositionHelper::bool_to_bitboard((position.wb & target_square_board) > 0) & target_square_board;
            position.wr ^= PositionHelper::bool_to_bitboard((position.wr & target_square_board) > 0) & target_square_board;
            position.wq ^= PositionHelper::bool_to_bitboard((position.wq & target_square_board) > 0) & target_square_board;

            position.move_number += 1;
        }

        position.en_passant_sq = PositionHelper::bool_to_bitboard(is_pawn_move && movement_sq_count == 16)
            & center_movement_sq_board;

        let not_fifty_move = (is_pawn_move || game_move.is_capture) as u8;
        position.fifty_move_count = (1 - not_fifty_move) * (position.fifty_move_count + 1);
        position.white_to_move = !position.white_to_move;
        position.update_occupancy();

        position.king_in_check = false;
        position.king_in_double_check = false;
        position.is_stalemate = false;
        position.is_checkmate = false;

        position.pin_ray_masks = [u64::MAX; 64];
        position.check_ray_mask =  u64::MAX;
    }

    pub fn unmake_move(&self, position: &mut Position, _game_move: &GameMove) {
        // Do this first so that the white vs. black logic below aligns with that above
        position.white_to_move = !position.white_to_move;

        position.wp = self.old_wp;
        position.wn = self.old_wn;
        position.wb = self.old_wb;
        position.wr = self.old_wr;
        position.wq = self.old_wq;
        position.wk = self.old_wk;

        position.bp = self.old_bp;
        position.bn = self.old_bn;
        position.bb = self.old_bb;
        position.br = self.old_br;
        position.bq = self.old_bq;
        position.bk = self.old_bk;

        position.en_passant_sq = self.old_en_passant_sq;
        position.castling_rights = self.old_castling_rights;

        position.move_number -= (!position.white_to_move) as u16;
        position.fifty_move_count = self.old_fifty_move_count;
        position.update_occupancy();

        position.king_in_check = self.old_king_in_check;
        position.king_in_double_check = self.old_king_in_double_check;
        position.is_stalemate = self.old_is_stalemate;
        position.is_checkmate = self.old_is_checkmate;

        position.pin_ray_masks = self.old_pin_ray_masks.clone();
        position.check_ray_mask =  self.old_check_ray_mask;
    }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayString;
    use crate::game::pieces::king::King;
    use crate::game::pieces::piece::Piece;
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
           extended_move_san: ArrayString::new()
        }, false);
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
            extended_move_san: ArrayString::new()
        }, false);
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
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wq, vec!["c5"]),
            (position.bp, vec!["a7", "b7", "c6", "e7", "f7", "g6", "h7", "h3"])
        ]);
    }

    #[test]
    fn test_make_move_2() {
        let (enemy_attacked_squares, mut position, mut move_list, mut king_attack_analyzer, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 1 1"));
        // g3+
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.wk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            &move_list,
            vec!["a6", "a4", "b4", "b5", "b6"],
            "a5a4 a5a6"
        );
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 14, // g2
            target_square: 22, // g3
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.bp, vec!["c7", "d6", "f4"]),
            (position.wp, vec!["e2", "g3", "b5"])
        ]);


        // Kxg3
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::KING,
            source_square: 31, // h4
            target_square: 22, // g3
            promotion_piece: PieceType::NONE,
            is_capture: true,
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.bp, vec!["c7", "d6", "f4"]),
            (position.wp, vec!["e2", "b5"])
        ]);
        move_list.clear();
        let enemy_attacked_squares = LegalMovesTestHelper::calc_enemy_attacked_squares(&mut position, &mut king_attack_analyzer);
        LegalMovesTestHelper::check_attack_and_movement_squares(
            King::calc_movements(&position, position.wk, &mut move_list, enemy_attacked_squares, &mut king_attack_analyzer),
            &move_list,
            vec!["a6", "a4", "b4", "b5", "b6"],
            "a5a4 a5a6"
        );
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
            extended_move_san: ArrayString::new()
        }, false);
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
            extended_move_san: ArrayString::new()
        }, false);
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
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wp, vec!["a2", "c4", "f2", "g2", "c6", "e5"]),
            (position.en_passant_sq, vec!["c3"])
        ]);
    }

    #[test]
    fn test_promotion() {
        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/ppP1ppbp/5np1/3PP1B1/8/Q5np/P1P2PP1/R3KB1R w Q - 1 2"));
        // c8=N
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 50, // c7
            target_square: 58, // c8
            promotion_piece: PieceType::KNIGHT,
            is_capture: false,
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wp, vec!["a2", "c2", "f2", "g2", "d5", "e5"]),
            (position.wn, vec!["c8"]),
            (position.bq, vec!["d8"])
        ]);


        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/ppP1ppbp/5np1/3PP1B1/8/Q5np/P1P2PP1/R3KB1R w Q - 1 2"));
        // cxd8=Q
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 50, // c7
            target_square: 59, // d8
            promotion_piece: PieceType::QUEEN,
            is_capture: true,
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wp, vec!["a2", "c2", "f2", "g2", "d5", "e5"]),
            (position.wq, vec!["a3", "d8"]),
            (position.bq, vec![])
        ]);
    }

    #[test]
    fn test_castling() {
        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/ppP1ppbp/5np1/3PP1B1/8/Q5np/P1P2PP1/R3K2R w KQ - 1 2"));
        // 1. o-o-o
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::KING,
            source_square: 4, // e1
            target_square: 2, // c1
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wk, vec!["c1"]),
            (position.wr, vec!["d1", "h1"]),
        ]);
        assert_eq!(position.castling_rights, 0);


        // 2. Capture one of the rooks
        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/ppP1ppbp/5np1/3PP1B1/8/Q5np/P1P2PP1/R3K2R b KQ - 1 2"));
        // Nxh1 (for black)
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::KNIGHT,
            source_square: 22, // g3
            target_square: 7, // h1
            promotion_piece: PieceType::NONE,
            is_capture: true,
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wk, vec!["e1"]),
            (position.wr, vec!["a1"]),
            (position.bn, vec!["f6", "h1"]),
        ]);
        // Only kingside castling should be removed
        assert_eq!(position.castling_rights, PositionHelper::bitboard_from_algebraic(vec!["c1"]));


        // 3. Move queenside rook and ensure that only queenside castling is removed
        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/ppP1ppbp/5np1/3PP1B1/8/Q5np/P1P2PP1/R3K2R w KQ - 1 2"));
        // Nxh1 (for black)
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::ROOK,
            source_square: 0, // a1
            target_square: 1, // b1
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        }, false);
        MoveMakerTestHelper::check_make_move_result(vec![
            (position.wk, vec!["e1"]),
            (position.wr, vec!["b1", "h1"]),
        ]);
        // Only queenside castling should be removed
        assert_eq!(position.castling_rights, PositionHelper::bitboard_from_algebraic(vec!["g1"]));
    }

    #[test]
    fn test_fifty_move_count() {
        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/ppP1ppbp/5np1/3PP1B1/8/Q5np/P1P2PP1/R3K2R w KQ - 1 2"));
        // 1. o-o-o ...
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::KING,
            source_square: 4, // e1
            target_square: 2, // c1
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        }, false);
        assert_eq!(position.fifty_move_count, 2);

        // 1. ... Nxh1 (capture one of the rooks)
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::KNIGHT,
            source_square: 22, // g3
            target_square: 7, // h1
            promotion_piece: PieceType::NONE,
            is_capture: true,
            extended_move_san: ArrayString::new()
        }, false);
        assert_eq!(position.fifty_move_count, 0);

        // 2. d6 ...
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 35, // d5
            target_square: 43, // d6
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        }, false);
        assert_eq!(position.fifty_move_count, 0);

        // 2. ... exd6
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::PAWN,
            source_square: 52, // d5
            target_square: 43, // d6
            promotion_piece: PieceType::NONE,
            is_capture: true,
            extended_move_san: ArrayString::new()
        }, false);
        assert_eq!(position.fifty_move_count, 0);

        // 3. Qb3
        move_maker.make_move(&mut position, &GameMove {
            piece: PieceType::QUEEN,
            source_square: 16, // d5
            target_square: 17, // d6
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        }, false);
        assert_eq!(position.fifty_move_count, 1);
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
            extended_move_san: ArrayString::new()
        });

        // Qxc5
        MoveMakerTestHelper::test_unmake_move(&mut position, &mut move_maker, &GameMove {
            piece: PieceType::QUEEN,
            source_square: 16, // a3
            target_square: 34, // c5
            promotion_piece: PieceType::NONE,
            is_capture: true,
            extended_move_san: ArrayString::new()
        });

        // d6
        MoveMakerTestHelper::test_unmake_move(&mut position, &mut move_maker, &GameMove {
            piece: PieceType::BISHOP,
            source_square: 5, // f1
            target_square: 12, // e2
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        });

        // o-o-o
        MoveMakerTestHelper::test_unmake_move(&mut position, &mut move_maker, &GameMove {
            piece: PieceType::KING,
            source_square: 4, // e1
            target_square: 2, // c1
            promotion_piece: PieceType::NONE,
            is_capture: false,
            extended_move_san: ArrayString::new()
        });

        let (_, mut position, _, _, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/ppP1ppbp/5np1/3PP1B1/8/Q5np/P1P2PP1/R3KB1R w Q - 1 2"));
        // cxd8=Q
        MoveMakerTestHelper::test_unmake_move(&mut position, &mut move_maker, &GameMove {
            piece: PieceType::PAWN,
            source_square: 50, // c7
            target_square: 59, // d8
            promotion_piece: PieceType::QUEEN,
            is_capture: true,
            extended_move_san: ArrayString::new()
        });
    }
}