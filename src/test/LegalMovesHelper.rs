use crate::constants::PlayerColour;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::moves::movemaker::MoveMaker;
use crate::game::pieces::king::King;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct LegalMovesTestHelper {

}

impl LegalMovesTestHelper {
    pub fn init_test_position_from_fen_str(position_str: Option<&str>) -> (u64, Position, GameMoveList, KingAttackRayAnalyzer, MoveMaker) {
        let move_maker = MoveMaker::default();
        let move_list = GameMoveList::default();
        let mut position = Position::from_fen(position_str, true).unwrap();
        let (enemy_attacked_squares, king_attack_analyzer) = Self::init_test_position_from_position(&mut position);
        (enemy_attacked_squares, position, move_list, king_attack_analyzer, move_maker)
    }

    pub fn init_test_position_from_position(position: &mut Position) -> (u64, KingAttackRayAnalyzer) {
        let mut king_attack_analyzer = KingAttackRayAnalyzer::default();
        let enemy_attacked_squares = Self::calc_enemy_attacked_squares(position, &mut king_attack_analyzer);
        // PositionAnalyzer::update_position_from_king_ray_attack_analysis(position, &king_attack_analyzer);

        (enemy_attacked_squares, king_attack_analyzer)
    }

    pub fn calc_enemy_attacked_squares(position: &mut Position, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        // let final_player_colour = player_colour.unwrap_or(if position.white_to_move { &PlayerColour::BLACK} else {&PlayerColour::WHITE});
        let final_player_colour = if position.white_to_move { &PlayerColour::BLACK} else {&PlayerColour::WHITE};
        let enemy_king_pos = match final_player_colour {
            PlayerColour::BLACK => position.wk,
            PlayerColour::WHITE => position.bk,
        };
        let attacked_squares = PositionAnalyzer::calc_all_attacked_squares(
            &position,
            final_player_colour,
            enemy_king_pos,
            king_attack_analyzer
        );
        PositionAnalyzer::update_position_from_king_ray_attack_analysis(position, &king_attack_analyzer);
        attacked_squares
    }

    pub fn check_attack_and_movement_squares(calc_movement_result: (u64, u64), expected_attack_squares: Vec<&str>, expected_movement_squares: Vec<&str>) {
        let (attacked_squares, movement_squares) = calc_movement_result;

        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(expected_attack_squares));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(expected_movement_squares));
    }

    pub fn generate_pin_ray_board(pin_rays: Vec<(&str, Vec<&str>)>) -> [u64; 64] {
        let mut result = [u64::MAX; 64];
        for pin_ray_tuple in pin_rays {
            let (board_sq, pin_ray) = pin_ray_tuple;
            result[PositionHelper::index_from_algebraic(board_sq) as usize] &= PositionHelper::bitboard_from_algebraic(pin_ray);
        }
        result
    }

    pub fn check_king_attack_analysis(king_attack_analyzer: &KingAttackRayAnalyzer, expected_pin_ray_masks: [u64; 64], expected_check_ray_mask: u64, expected_num_checking_pieces: u8, expected_disable_en_passant: bool) {
        assert_eq!(king_attack_analyzer.pin_ray_masks, expected_pin_ray_masks);
        assert_eq!(king_attack_analyzer.check_ray_mask, expected_check_ray_mask);
        assert_eq!(king_attack_analyzer.num_checking_pieces, expected_num_checking_pieces);
        assert_eq!(king_attack_analyzer.disable_en_passant, expected_disable_en_passant);
    }

    pub fn switch_sides(position: &mut Position, move_list: Option<&mut GameMoveList>, king_attack_analyzer: Option<&mut KingAttackRayAnalyzer>) -> u64 {
        position.white_to_move = !position.white_to_move;
        position.update_occupancy();
        if move_list.is_some() { move_list.unwrap().clear(); }

        let mut temp_king_attack_analyzer = KingAttackRayAnalyzer::default();
        let final_king_attack_analyzer = king_attack_analyzer.unwrap_or(&mut temp_king_attack_analyzer);
        final_king_attack_analyzer.clear();

        Self::calc_enemy_attacked_squares(position, final_king_attack_analyzer)
    }
}