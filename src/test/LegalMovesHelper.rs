use crate::constants::PlayerColour;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::gamemove::GameMove;
use crate::game::gamemovelist::GameMoveList;
use crate::game::pieces::king::King;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;

pub struct LegalMovesHelper {

}

impl LegalMovesHelper {
    pub fn init_test_position_from_fen_str(position_str: Option<&str>) -> (u64, Position, GameMoveList, KingAttackRayAnalyzer) {
        let move_list = GameMoveList::default();
        let position = Position::from_fen(position_str).unwrap();
        let (enemy_attacked_squares, king_attack_analyzer) = Self::init_test_position_from_position(&position);
        (enemy_attacked_squares, position, move_list, king_attack_analyzer)
    }

    pub fn init_test_position_from_position(position: &Position) -> (u64, KingAttackRayAnalyzer) {
        let mut king_attack_analyzer = KingAttackRayAnalyzer::default();
        let enemy_attacked_squares = Self::calc_enemy_attacked_squares(&position, &mut king_attack_analyzer);

        (enemy_attacked_squares, king_attack_analyzer)
    }

    pub fn calc_enemy_attacked_squares(position: &Position, king_attack_analyzer: &mut KingAttackRayAnalyzer) -> u64 {
        // let final_player_colour = player_colour.unwrap_or(if position.white_to_move { &PlayerColour::BLACK} else {&PlayerColour::WHITE});
        let final_player_colour = if position.white_to_move { &PlayerColour::BLACK} else {&PlayerColour::WHITE};
        let enemy_king_pos = match final_player_colour {
            PlayerColour::BLACK => position.wk,
            PlayerColour::WHITE => position.bk,
        };
        PositionAnalyzer::calc_all_attacked_squares(
            &position,
            final_player_colour,
            enemy_king_pos,
            king_attack_analyzer
        )
    }

    pub fn check_attack_and_movement_squares(calc_movement_result: (u64, u64), expected_attack_squares: Vec<&str>, expected_movement_squares: Vec<&str>) {
        let (attacked_squares, movement_squares) = calc_movement_result;

        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(expected_attack_squares));
        assert_eq!(movement_squares, PositionHelper::bitboard_from_algebraic(expected_movement_squares));
    }

    pub fn generate_pin_ray_board(pin_rays: Vec<(usize, Vec<&str>)>) -> [u64; 64] {
        let mut result = [0u64; 64];
        for pin_ray_tuple in pin_rays {
            let (sq_ind, pin_ray) = pin_ray_tuple;
            result[sq_ind] = PositionHelper::bitboard_from_algebraic(pin_ray);
        }
        result
    }

    pub fn check_king_attack_analysis(king_attack_analyzer: &KingAttackRayAnalyzer, expected_pin_rays: [u64; 64], expected_check_rays: u64, expected_num_checking_pieces: u8, expected_disable_en_passant: bool) {
        assert_eq!(king_attack_analyzer.pin_rays, expected_pin_rays);
        assert_eq!(king_attack_analyzer.check_rays, expected_check_rays);
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