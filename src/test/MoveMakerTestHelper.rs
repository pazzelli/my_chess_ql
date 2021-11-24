use crate::constants::PieceType;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::movemaker::MoveMaker;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct MoveMakerTestHelper {

}

impl MoveMakerTestHelper {
    pub fn check_make_move_result(expected_results: Vec<(u64, Vec<&str>)>) {
        // let (_, mut position, mut move_list, mut king_attack_analyzer, mut move_maker) = LegalMovesTestHelper::init_test_position_from_fen_str(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/3RKB1R w - - 1 2"));
        // move_maker.make_move(position, game_move);

        for (board, target_squares) in expected_results {
            assert_eq!(board, PositionHelper::bitboard_from_algebraic(target_squares));
        }
    }

    pub fn test_unmake_move(position: &mut Position, move_maker: &mut MoveMaker, game_move: &GameMove) {
        let orig = position.clone();

        move_maker.make_move(position, game_move);
        move_maker.unmake_move(position, game_move);

        assert_eq!(position.wp, orig.wp);
        assert_eq!(position.wn, orig.wn);
        assert_eq!(position.wb, orig.wb);
        assert_eq!(position.wr, orig.wr);
        assert_eq!(position.wq, orig.wq);
        assert_eq!(position.wk, orig.wk);
        assert_eq!(position.bq, orig.bq);
        assert_eq!(position.bn, orig.bn);
        assert_eq!(position.bb, orig.bb);
        assert_eq!(position.br, orig.br);
        assert_eq!(position.bq, orig.bq);
        assert_eq!(position.bk, orig.bk);
        assert_eq!(position.en_passant_sq, orig.en_passant_sq);
        assert_eq!(position.castling_rights, orig.castling_rights);
        assert_eq!(position.white_occupancy, orig.white_occupancy);
        assert_eq!(position.black_occupancy, orig.black_occupancy);
        assert_eq!(position.friendly_occupancy, orig.friendly_occupancy);
        assert_eq!(position.enemy_occupancy, orig.enemy_occupancy);
        assert_eq!(position.non_occupancy, orig.non_occupancy);
        assert_eq!(position.all_occupancy, orig.all_occupancy);
        assert_eq!(position.white_to_move, orig.white_to_move);
        assert_eq!(position.fifty_move_count, orig.fifty_move_count);
        assert_eq!(position.move_number, orig.move_number);

    }
}