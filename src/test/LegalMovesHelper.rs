
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;
use std::process::*;
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


#[cfg(test)]
mod tests {
    use std::borrow::{Borrow, BorrowMut};
    use std::fs::File;
    use std::io;
    use std::io::{BufRead, BufReader, Read, Stderr, Write};
    use std::path::Path;
    use std::time::Instant;
    use json::JsonValue;
    use simple_error::SimpleError;
    use crate::constants::PieceType;
    use crate::game::analysis::positionanalyzer::PositionAnalyzer;
    use crate::game::moves::gamemovelist::GameMoveList;
    use crate::game::moves::movemaker::MoveMaker;
    use crate::test::legalmoveshelper::LegalMovesTestHelper;
    use crate::test::movemakertesthelper::MoveMakerTestHelper;

    use super::*;

    fn compare_move_lists(position: &Position, move_list: &GameMoveList, temp_pos: &Position, temp_move_list: &GameMoveList) -> Result<(), SimpleError> {
        let move_list_str = format!("{:?}", move_list);
        let temp_move_list_str = format!("{:?}", temp_move_list);

        if move_list_str != temp_move_list_str {
            println!("MISMATCH!!!");
            PositionHelper::print_position(position);
            println!();
            PositionHelper::print_position(&temp_pos);
            println!();
            println!("{}", position.to_fen());
            println!("{}", temp_pos.to_fen());
            println!("{}", move_list_str);
            println!("{}", temp_move_list_str);

            MoveMakerTestHelper::compare_positions(position, &temp_pos);
            return Err(SimpleError::new("Move list mismatch"));
        }
        Ok(())
    }

    fn test_perft_recursive(position: &mut Position, depth: u8, intense_verify: bool, stockfish: &mut Option<Child>) -> Result<usize, SimpleError> {
        if depth <= 0 {  return Ok(1); }

        let mut move_maker = MoveMaker::default();
        let mut move_list = GameMoveList::default();
        PositionAnalyzer::calc_legal_moves(position, &mut move_list);

        if intense_verify {
            let fen = position.to_fen();
            let mut temp_pos = Position::from_fen(Some(fen.as_str()), false).unwrap();
            let mut temp_move_list = GameMoveList::default();
            PositionAnalyzer::calc_legal_moves(&mut temp_pos, &mut temp_move_list);

            // Ensure move lists from the current / reused Position object match with that generated from a fresh one
            compare_move_lists(&position, &move_list, &temp_pos, &temp_move_list)?;

            if stockfish.is_some() {
                // let (sf_moves, sf_move_count) = run_stockfish_analysis(fen.as_str(), stockfish.as_mut().unwrap());
                let sf_moves = run_stockfish_analysis(fen.as_str(), stockfish.as_mut().unwrap());
                let move_list_str = format!("{:?}", move_list);

                // if sf_move_count != move_list.list_len || move_list_str != sf_moves {
                if move_list_str != sf_moves {
                    println!("{}", "STOCKFISH MISMATCH!!!");
                    println!();
                    println!("Failing position:\t{}", position.to_fen());
                    println!("My moves:\t{}", move_list_str);
                    println!("Stockfish:\t{}", sf_moves);
                    return Err(SimpleError::new("Stockfish move list mismatch"));
                }
                // else {
                //     println!("{}", sf_moves.as_str());
                // }
            }
        }

        // if depth <= 1 {
        //     return Ok(move_list.list_len);
        // }

        let mut nodes = 0;
        for i in 0..move_list.list_len {
            move_maker.make_move(position, &move_list.move_list[i]);

            match self::test_perft_recursive(position, depth - 1, intense_verify, stockfish) {
                Ok(n) => nodes += n,
                Err(e) => { println!("Last move played: {:?}", move_list.move_list[i]); return Err(e); }
            }

            move_maker.unmake_move(position, &move_list.move_list[i]);

        }
        Ok(nodes)
    }

    #[test]
    fn test_perft() {
        // Tests just a basic perft run from the starting position, with the depth specified
        let depth: u8 = 1;
        let verify_with_stockfish = false;

        // Starting position
        let mut stockfish = if verify_with_stockfish { open_stockfish() } else { None };
        let mut position = Position::from_fen(None, true).unwrap();

        match test_perft_recursive(&mut position, depth, true, &mut stockfish){
            Ok(result) => println!("Depth: {}\tNodes: {:}", depth, result),
            Err(_) => ()
        };

        // Original algorithm in C from chess programming wiki
        // u64 Perft(int depth /* assuming >= 1 */)
        // {
        //     MOVE move_list[256];
        //     int n_moves, i;
        //     u64 nodes = 0;
        //
        //     n_moves = GenerateLegalMoves(move_list);
        //
        //     if (depth == 1)
        //     return (u64) n_moves;
        //
        //     for (i = 0; i < n_moves; i++) {
        //         MakeMove(move_list[i]);
        //         nodes += Perft(depth - 1);
        //         UndoMove(move_list[i]);
        //     }
        //     return nodes;
        // }
    }

    #[test]
    fn run_legal_moves_test_cases() {
        // Set this to true to use Stockfish to verify generated moves
        let intense_verify = false;
        let mut stockfish = if intense_verify { None } else { open_stockfish() };
        let json_data = read_legal_moves_test_cases();

        for test_case in json_data.members() {
            let depth: u8 = test_case["depth"].as_u8().unwrap();
            let expected_nodes: usize = test_case["nodes"].as_usize().unwrap();
            let fen: &str = test_case["fen"].as_str().unwrap();

            let mut position = Position::from_fen(Some(fen), true).unwrap();

            let before = Instant::now();
            let node_count =
                match test_perft_recursive(&mut position, depth, intense_verify, &mut stockfish) {
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

    fn run_stockfish_analysis(fen: &str, stockfish: &mut Child) -> String {
        let mut stockfish_in = stockfish.stdin.as_mut().unwrap();
        let mut stockfish_out = BufReader::new(stockfish.stdout.as_mut().unwrap());

        let commands = format!("position fen {}\ngo perft 1\n", fen);

        write_to_stockfish(&mut stockfish_in, commands.as_str());
        let (moves, _move_count) = read_from_stockfish(&mut stockfish_out);
        moves
    }

    fn open_stockfish() -> Option<Child> {
        // Open stockfish and pipe its inputs and outputs so they can be read/written to from here
        let stockfish = Command::new("/usr/local/bin/stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn();

        match stockfish.is_err() {
            true => { println!("Warning: unable to open stockfish: {}\nStockfish move verification will be disabled", stockfish.err().unwrap()); None }
            false => {
                println!("{}", "Stockfish was opened");
                // Ensure Stockfish is ready
                std::thread::sleep(std::time::Duration::from_millis(1500));
                Some(stockfish.unwrap())
            }
        }
    }

    fn write_to_stockfish(stockfish_in: &mut ChildStdin, commands: &str) {
        // println!("{}", commands);
        stockfish_in.write(commands.as_bytes()).unwrap();
        // stockfish_in.flush();
    }

    fn read_from_stockfish(stockfish_out: &mut BufReader<&mut ChildStdout>) -> (String, usize) {
        let mut moves = Vec::<String>::with_capacity(256);

        lazy_static! {
            static ref RE: Regex = Regex::new(r"^[a-h][1-8][a-h][1-8][nrqb]?").unwrap();
        }

        loop {
            let mut line = String::new();
            stockfish_out.read_line(&mut line).unwrap();
            // print!("{}", line);

            let mtch = RE.find(line.as_str());
            if mtch.is_some() {
                moves.push(String::from(mtch.unwrap().as_str()));
            }

            if line.contains("Nodes searched") { moves.sort(); return (moves.join(" "), moves.len()) }
        }
    }
}