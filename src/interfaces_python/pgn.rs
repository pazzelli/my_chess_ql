use std::collections::VecDeque;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};

use regex::Regex;
use rand::prelude::*;
use simple_error::{bail, SimpleError};
use crate::constants::*;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::moves::movemaker::MoveMaker;
use crate::game::position::Position;
use crate::neural::positionconverter::*;

const MIN_ELO_RATING: i16 = 2200;

pub struct PGNReader {
    file: BufReader<File>,
    buf: Vec<u8>,
    pgn_game_position: Position,
    position_moves: GameMoveList,
    pgn_next_move_played: GameMove,
    pgn_game_moves: VecDeque<String>,
    pgn_game_result: f32,
    move_maker: MoveMaker,
    white_min_elo: bool,
    black_min_elo: bool,
}

impl PGNReader {
    pub fn init_pgn_file(file_path: &str) -> Self {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(file_path);
        let display = path.display();

        // Open the path in read-only mode, returns `io::Result<File>`
        let file = match File::open(&path) {
            Err(why) => panic!("Couldn't open PGN file {}: {}", display, why),
            Ok(file) => file,
        };

        PGNReader {
            file: BufReader::with_capacity(2048,  file),
            buf: Vec::<u8>::with_capacity(2048),
            pgn_game_position: Position::from_fen(None, false).unwrap(),
            position_moves: GameMoveList::default(),
            pgn_next_move_played: GameMove::default(),
            pgn_game_moves: VecDeque::with_capacity(256),
            pgn_game_result: 0.0,
            move_maker: MoveMaker::default(),
            white_min_elo: false,
            black_min_elo: false,
        }
    }

    fn get_next_pgn_line(&mut self) -> Option<String> {
        // // Can't use a straightforward File::read_line() approach since it enforces strict conversion
        // // to UTF-8 and this causes errors due to special characters in some player names
        self.buf.clear();

        loop {
            match self.file.read_until(b'\n', &mut self.buf) {
                Ok(n) => {
                    if n <= 0 { return None }

                    return Some(String::from(String::from_utf8_lossy(&self.buf).trim()));
                },
                Err(e) => { println!("Error reading from PGN file: {}", e); return None }
            }
        }
    }

    fn parse_pgn_header(header_line: &str) -> Result<(String, String), SimpleError> {
        let header_line = String::from(header_line);

        // Header lines are enclosed in square brackets so iterate such that they are ignored
        for i in 1..header_line.len() - 1 {
            // Split header lines at a blank space
            // Header values are enclosed in double quotes, so ignore those quotes
            if &header_line[i..i+1] == " " {
                let (key, value) = (&header_line[1..i], &header_line[i+1..]);
                return Ok((String::from(key), String::from(&value[1..value.len() - 2])));
            }
        }
        bail!("Invalid header record {}", header_line)    // record was invalid
    }

    fn parse_pgn_game_moves(game_moves_line: &str, game_result: &str, game_moves: &mut VecDeque<String>) {
        let game_moves_line = String::from(game_moves_line);

        // Split game moves into a vector, ignoring any comments / commentary
        let split_moves = game_moves_line.split(" ");
        let mut commentary_brackets_found = 0u8;
        for candidate_move_token in split_moves {
            if candidate_move_token.len() <= 0 || candidate_move_token == game_result || candidate_move_token == "..." { continue; };

            // Comments continue to end of the line
            if candidate_move_token.starts_with(";") { return; }

            // Not sure why tokens such as "$4" or "$138" within the game moves are present, but if so, remove them
            if candidate_move_token.starts_with("$") { continue; }

            // Track start of a commentary block
            if candidate_move_token.starts_with("{") || candidate_move_token.starts_with("(") {
                commentary_brackets_found += 1;
            }

            // Ensure to catch the end of a commentary block before skipping this token (if we're
            // already within a block)
            if candidate_move_token.ends_with("}") || candidate_move_token.ends_with(")") {
                commentary_brackets_found -= 1;
                continue;
            }

            // Skip any commentary blocks
            if commentary_brackets_found > 0 { continue; }

            // Skip move numbers - this is much faster than using the GAME_MOVE_NUMBER regex
            let is_move_number = match candidate_move_token.chars().next().unwrap() {
                '0'..='9' => true,
                _ => false
            };
            if is_move_number { continue; };
            // if GAME_MOVE_NUMBER.find(candidate_move_token).is_some() { continue; }

            // If we made it here, we should finally be looking at a valid move token
            game_moves.push_back(String::from(candidate_move_token.trim()));
        }
    }

    fn parse_pgn_game_result(game_result: &str) -> f32 {
        // Not sure if I should really be assuming a draw if there was no real game result specified
        match game_result {
            "0-1" => -1.0,
            "1-0" => 1.0,
            "1/2-1/2" => 0.,
            _ => 0.0
        }
    }

    fn get_next_pgn_game(&mut self) -> Option<(Position, VecDeque<String>, f32, bool, bool)> {
        // Loop over games
        loop {
            let (mut white_elo, mut black_elo) = (-1i16, -1i16);
            let mut position: Position = Position::from_fen(None, false).unwrap();
            let mut game_moves: VecDeque<String> = VecDeque::with_capacity(256);
            let mut game_moves_found = false;
            let mut game_result: String = String::from("");

            // Loop over lines within the games
            loop {
                let line: Option<String> = self.get_next_pgn_line();
                if line.is_none() { return None }
                let line = line.unwrap();

                if line.len() <= 0 {
                    // Empty line after a game means this game has ended
                    if game_moves_found { break; }

                    // Otherwise, just skip any blank lines
                    continue;

                } else if line.starts_with("[") {
                    // Check for header lines
                    let (key, value) = PGNReader::parse_pgn_header(line.as_str()).unwrap();

                    match key.as_str() {
                        "WhiteElo" => white_elo = value.parse::<i16>().unwrap(),
                        "BlackElo" => black_elo = value.parse::<i16>().unwrap(),
                        "FEN" => position = Position::from_fen(Some(value.as_str()), false).unwrap(),
                        "Result" => game_result = value,
                        _ => {} // don't care about any other headers for now
                    }

                } else if match line.chars().next().unwrap() {'0'..='9'=> true, _ => false } || game_moves_found {
                    // Once game moves are found, keep reading more lines until a blank is encountered
                    // Game moves
                    game_moves_found = true;
                    PGNReader::parse_pgn_game_moves(line.as_str(), game_result.as_str(), &mut game_moves);

                    // println!("{}", line);
                }
            }

            let white_min_elo = white_elo >= MIN_ELO_RATING;
            let black_min_elo = black_elo >= MIN_ELO_RATING;
            if (white_min_elo || black_min_elo) && game_moves.len() > 0 {
                return Some((position, game_moves, PGNReader::parse_pgn_game_result(game_result.as_str()), white_min_elo, black_min_elo));
            }
        }
    }

    fn set_next_pgn_move_played(&mut self) {
        let next_move_san = self.pgn_game_moves.pop_front().unwrap();

        let move_match = self.position_moves.get_move_by_partial_san(next_move_san.as_str());
        if move_match.is_none() {
            println!("{:?}", self.pgn_game_moves);
            panic!("{}", format!("Can't find source move {}", next_move_san.as_str()).as_str());
        }
        //.expect(format!("Can't find source move {}", next_move_san.as_str()).as_str());
        self.pgn_next_move_played = move_match.unwrap();
    }

    pub fn get_next_position(&mut self) -> Option<([u8; NN_PLANE_COUNT_GAME_PIECE_INPUTS << 6], [u8; NN_PLANE_COUNT_AUX_INPUTS << 6], [u8; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6], [u8; NN_PLANE_COUNT_MOVEMENT_OUTPUTS << 6], f32)> {
        loop {
            // Need to load a new game if there are 1 or fewer moves remaining
            // The final move is not played since it does not have a next move (target move) to train on
            if self.pgn_game_moves.len() <= 1 {
                let (pgn_game_position, pgn_game_moves, game_result, white_min_elo, black_min_elo) = self.get_next_pgn_game()?;

                self.pgn_game_position = pgn_game_position;
                self.pgn_game_moves = pgn_game_moves;
                self.pgn_game_result = game_result;
                self.white_min_elo = white_min_elo;
                self.black_min_elo = black_min_elo;

            } else {
                // If we already have a position loaded and a valid next move, then make it!
                // (i.e. apply it to the current position)
                self.move_maker.make_move(&mut self.pgn_game_position, &self.pgn_next_move_played, false);
            }

            // Update list of available moves in the position
            self.position_moves.clear();
            PositionAnalyzer::calc_legal_moves(&mut self.pgn_game_position, &mut self.position_moves);

            // Keep track of the next move played in the game to make life easier
            self.set_next_pgn_move_played();

            // Return an encoded position for the neural network only if the current player has reached the min ELO cutoff
            if (self.pgn_game_position.white_to_move && self.white_min_elo) || (!self.pgn_game_position.white_to_move && self.black_min_elo) {
                let (input_piece_planes, input_aux_planes, output_move_planes) = PositionConverter::convert_position_for_nn(&self.pgn_game_position, &self.position_moves);
                let output_target_move_plane = PositionConverter::convert_target_move_for_nn(&self.pgn_next_move_played, !self.pgn_game_position.white_to_move);

                return Some((
                    input_piece_planes,
                    input_aux_planes,
                    output_move_planes,
                    output_target_move_plane,
                    self.pgn_game_result)
                );
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::time::Instant;
    use super::*;

    // const GAME_1: &str = "\
    //     [Event \"F/S Return Match\"]
    //     [Site \"Belgrade, Serbia JUG\"]
    //     [Date \"1992.11.04\"]
    //     [Round \"29\"]
    //     [White \"Fischer, Robert J.\"]
    //     [Black \"Spassky, Boris V.\"]
    //     [Result \"1/2-1/2\"]
    //
    //     1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 {This opening is called the Ruy Lopez.}
    //     4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O 9. h3 Nb8 10. d4 Nbd7
    //     11. c4 c6 12. cxb5 axb5 13. Nc3 Bb7 14. Bg5 b4 15. Nb1 h6 16. Bh4 c5 17. dxe5
    //     Nxe4 18. Bxe7 Qxe7 19. exd6 Qf6 20. Nbd2 Nxd6 21. Nc4 Nxc4 22. Bxc4 Nb6
    //     23. Ne5 Rae8 24. Bxf7+ Rxf7 25. Nxf7 Rxe1+ 26. Qxe1 Kxf7 27. Qe3 Qg5 28. Qxg5
    //     hxg5 29. b3 Ke6 30. a3 Kd6 31. axb4 cxb4 32. Ra5 Nd5 33. f3 Bc8 34. Kf2 Bf5
    //     35. Ra7 g6 36. Ra6+ Kc5 37. Ke1 Nf4 38. g3 Nxh3 39. Kd2 Kb5 40. Rd6 Kc5 41. Ra6
    //     Nf2 42. g4 Bd3 43. Re6 1/2-1/2
    // ";

    #[test]
    fn test_parse_pgn_header() {
        let (key, value) = PGNReader::parse_pgn_header("[White \"Fischer, Robert J.\"]").unwrap();
        assert_eq!(key, "White");
        assert_eq!(value, "Fischer, Robert J.");

        let (key, value) = PGNReader::parse_pgn_header("[WhiteElo \"2850\"]").unwrap();
        assert_eq!(key, "WhiteElo");
        assert_eq!(value, "2850");
    }

    fn test_parse_game_moves_helper(game_move_str: &str, game_result: &str) -> String {
        let mut game_moves: VecDeque<String> = VecDeque::with_capacity(256);
        PGNReader::parse_pgn_game_moves(game_move_str, game_result, &mut game_moves);
        format!("{:?}", game_moves)
    }

    #[test]
    fn test_parse_pgn_game_moves() {
        assert_eq! (
            self::test_parse_game_moves_helper("1. e4 e5 2. Nf3 Nc6 3. Bb5 a6 {This opening is called the Ruy Lopez.} 4. Ba4 Nf6 5. O-O Be7 6. Re1 b5 7. Bb3 d6 8. c3 O-O-O  1/2-1/2 ;9. h3 Nb8 10. d4 Nbd7", "1/2-1/2"),
            "[\"e4\", \"e5\", \"Nf3\", \"Nc6\", \"Bb5\", \"a6\", \"Ba4\", \"Nf6\", \"O-O\", \"Be7\", \"Re1\", \"b5\", \"Bb3\", \"d6\", \"c3\", \"O-O-O\"]"
        );

        assert_eq! (
            self::test_parse_game_moves_helper("1. e4 e5 2. O-O-O ...  0-1 ;a comment goes here", "0-1"),
            "[\"e4\", \"e5\", \"O-O-O\"]"
        );

        assert_eq! (
            self::test_parse_game_moves_helper("1. Nf3 Nf6 2. c4 e6 3. Nc3 Bb4 4. Qc2 O-O 5. a3 Bxc3 6. Qxc3 b6 7. e3 Bb7 8. Be2 d6 9. O-O Nbd7 10. b4 e5 11. Bb2 Re8 12. d3 c5 13. Rae1 Rc8 14. b5 d5 15. cxd5 Nxd5 16. Qb3 Qf6 17. Nd2 Nc7 18. f4 Qg6 19. e4 exf4 20. Rxf4 Ne6 21. Rf5 Qh6 22. Nc4 Nd4 23. Bxd4 cxd4 24. Rxf7 Nc5 25. Qa2 Bd5 26. Rxa7 ( 26. Rf2 ) 26... Be6 27. Bf1 Rf8 28. Qd2 Qh4 29. g3 Qd8 30. Ne5 Nb3 31. Qb2 Rc3 32. Nc6 Qg5 33. Ne7+ Kh8 34. Nd5 Nd2 35. Bg2 Rxd3 36. Nf4 Rxf4 37. gxf4 $4 ( 37. Ra8+ Bg8 38. Qa2 ) 37... Nf3+ 38. Kh1 Qh4 39. Ra8+ Bg8 40. Rxg8+ Kxg8  0-1", "0-1"),
            "[\"Nf3\", \"Nf6\", \"c4\", \"e6\", \"Nc3\", \"Bb4\", \"Qc2\", \"O-O\", \"a3\", \"Bxc3\", \"Qxc3\", \"b6\", \"e3\", \"Bb7\", \"Be2\", \"d6\", \"O-O\", \"Nbd7\", \"b4\", \"e5\", \"Bb2\", \"Re8\", \"d3\", \"c5\", \"Rae1\", \"Rc8\", \"b5\", \"d5\", \"cxd5\", \"Nxd5\", \"Qb3\", \"Qf6\", \"Nd2\", \"Nc7\", \"f4\", \"Qg6\", \"e4\", \"exf4\", \"Rxf4\", \"Ne6\", \"Rf5\", \"Qh6\", \"Nc4\", \"Nd4\", \"Bxd4\", \"cxd4\", \"Rxf7\", \"Nc5\", \"Qa2\", \"Bd5\", \"Rxa7\", \"Be6\", \"Bf1\", \"Rf8\", \"Qd2\", \"Qh4\", \"g3\", \"Qd8\", \"Ne5\", \"Nb3\", \"Qb2\", \"Rc3\", \"Nc6\", \"Qg5\", \"Ne7+\", \"Kh8\", \"Nd5\", \"Nd2\", \"Bg2\", \"Rxd3\", \"Nf4\", \"Rxf4\", \"gxf4\", \"Nf3+\", \"Kh1\", \"Qh4\", \"Ra8+\", \"Bg8\", \"Rxg8+\", \"Kxg8\"]"
        );

        assert_eq! (
            self::test_parse_game_moves_helper("1. e4 c5 2. Nf3 e6 3. b3 Nc6 4. Bb2 d5 5. exd5 exd5 6. Bb5 Nf6 7. O-O Be7 8. Ne5 Qc7 9. Re1 O-O 10. Nxc6 bxc6 11. Be2 Ne4 12. Bf3 f5 13. d3 Bd6 14. dxe4 Bxh2+ 15. Kh1 Qf4 ( 15... fxe4 16. Bxe4 dxe4 17. Nd2 Qd6 18. Qh5 ) 16. exd5 ( 16. g3 Qh6 17. Bc1 f4 18. Bh5 Qe6 19. Bg4 ( 19. g4 Qf6 20. Kxh2 Qh4+ ( 20... Qxa1 ) 21. Kg1 f3 22. Qd3 Bxg4 23. exd5 ) 19... Qe5 20. Bxc8 fxg3 21. exd5 Qxa1 22. Be6+ Kh8 23. Be3 gxf2 24. Rf1 ) 16... Qh4 17. Na3 Be5+ 18. Kg1 Qh2+ 19. Kf1 Bxb2 20. Nc4 Bxa1 21. Qxa1 Qh1+ 22. Ke2 Re8+ 23. Kd2 Qh6+ 24. Re3 Bb7 25. d6 Rad8 26. Qe1 Ba6 27. Kc3 Qf6+ 28. Kd2 Qd4+  0-1", "0-1"),
            "[\"e4\", \"c5\", \"Nf3\", \"e6\", \"b3\", \"Nc6\", \"Bb2\", \"d5\", \"exd5\", \"exd5\", \"Bb5\", \"Nf6\", \"O-O\", \"Be7\", \"Ne5\", \"Qc7\", \"Re1\", \"O-O\", \"Nxc6\", \"bxc6\", \"Be2\", \"Ne4\", \"Bf3\", \"f5\", \"d3\", \"Bd6\", \"dxe4\", \"Bxh2+\", \"Kh1\", \"Qf4\", \"exd5\", \"Qh4\", \"Na3\", \"Be5+\", \"Kg1\", \"Qh2+\", \"Kf1\", \"Bxb2\", \"Nc4\", \"Bxa1\", \"Qxa1\", \"Qh1+\", \"Ke2\", \"Re8+\", \"Kd2\", \"Qh6+\", \"Re3\", \"Bb7\", \"d6\", \"Rad8\", \"Qe1\", \"Ba6\", \"Kc3\", \"Qf6+\", \"Kd2\", \"Qd4+\"]"
        );
    }

    #[test]
    fn test_read_pgn_test_file() {
        // let file_path = "src/test/resources/bundesliga2000.pgn";
        let file_path = "src/test/resources/test_pgn1.pgn";
        let mut pgn = PGNReader::init_pgn_file(file_path);
        let mut total_positions = 0;
        println!("Initializing from file: {}", file_path);

        let before = Instant::now();

        loop {
            let x = pgn.get_next_position();
            if x.is_none() { break; }

            if total_positions % 100000 == 0 {
                println!("Total positions: {}", total_positions);
                let elapsed = before.elapsed();
                println!("Elapsed time: {:.2?}  ({:.1?} pos/s)", elapsed, (total_positions as f64 / elapsed.as_millis() as f64) * 1000f64);
                std::io::stdout().flush().unwrap_or(());
            }

            total_positions += 1;
        }

        println!("Total positions: {}", total_positions);
        let elapsed = before.elapsed();
        println!("Elapsed time: {:.2?}  ({:.1?} pos/s)", elapsed, (total_positions as f64 / elapsed.as_millis() as f64) * 1000f64);
    }
}

