
use regex::Regex;
use std::path::PathBuf;
use std::process::*;
use crate::constants::PlayerColour;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::moves::movemaker::MoveMaker;
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

    pub fn check_attack_and_movement_squares(calc_movement_result: (u64, u64), move_list: &GameMoveList, expected_attack_squares: Vec<&str>, expected_movement_square_list_str: &str) {
        let (attacked_squares, _movement_squares) = calc_movement_result;

        assert_eq!(attacked_squares, PositionHelper::bitboard_from_algebraic(expected_attack_squares));
        assert_eq!(format!("{:?}", move_list), expected_movement_square_list_str);
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


#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::time::Instant;
    use json::JsonValue;
    use simple_error::SimpleError;
    use crate::benchmarks::perftbenchmark::PerftBenchmark;
    use crate::game::analysis::positionanalyzer::PositionAnalyzer;
    use crate::game::moves::gamemovelist::GameMoveList;
    use crate::game::moves::movemaker::MoveMaker;
    use crate::interfaces::stockfish::StockfishInterface;
    use crate::test::movemakertesthelper::MoveMakerTestHelper;

    use super::*;

    #[test]
    fn run_legal_moves_test_cases() {
        // Set this to true to use Stockfish to verify generated moves
        let intense_verify = false;
        let mut stockfish = if intense_verify { StockfishInterface::open_stockfish() } else { None };
        let json_data = read_legal_moves_test_cases();

        for test_case in json_data.members() {
            let depth: u8 = test_case["depth"].as_u8().unwrap();
            let expected_nodes: usize = test_case["nodes"].as_usize().unwrap();
            let fen: &str = test_case["fen"].as_str().unwrap();

            if !intense_verify && expected_nodes > 500000 { continue; }

            let mut position = Position::from_fen(Some(fen), true).unwrap();

            let before = Instant::now();
            let node_count =
                match PerftBenchmark::run_perft_recursive(&mut position, depth, intense_verify, &mut stockfish) {
                    Ok(result) => result,
                    Err(_) => 0
                };
            let elapsed = before.elapsed();
            println!("Elapsed time: {:.2?}  ({:.1?} pos/s)", elapsed, (node_count as f64 / elapsed.as_millis() as f64) * 1000f64);

            println!("Depth: {}\tNodes: {}\tExpected: {}\n", depth, node_count, expected_nodes);
            assert_eq!(node_count, expected_nodes);
        }
    }

    fn read_legal_moves_test_cases() -> JsonValue {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/test/resources/TestPositions.json");
        let display = path.display();

        // Open the path in read-only mode, returns `io::Result<File>`
        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why),
            Ok(file) => file,
        };

        // Read the file contents into a string, returns `io::Result<usize>`
        let mut s = String::new();
        match file.read_to_string(&mut s) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(_) => () //print!("{} contains:\n{}", display, s),
        }

        json::parse(s.as_str()).unwrap()
    }
}