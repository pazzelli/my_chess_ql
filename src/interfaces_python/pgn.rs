use std::collections::VecDeque;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};

use regex::Regex;
use rand::prelude::*;
use simple_error::{bail, SimpleError};
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::moves::movemaker::MoveMaker;
use crate::game::position::Position;

const MIN_ELO_RATING: i16 = 2200;

lazy_static! {
    static ref GAME_MOVE_NUMBER: Regex = Regex::new(r"^[0-9]+\.").unwrap();
    static ref GAME_MOVE_EXTENDED_SAN: Regex = Regex::new(r"^([BNKQRP]?)([a-h]?)([1-8]?)(x?)([a-h][1-8])(=[BNQR])?").unwrap();
}

pub struct PGNReader {
    file: BufReader<File>,
    buf: Vec<u8>,
    pgn_game_position: Position,
    position_moves: GameMoveList,
    pgn_game_moves: VecDeque<String>,
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
            pgn_game_moves: VecDeque::with_capacity(256),
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
        let mut commentary_found = false;
        for candidate_move_token in split_moves {
            if candidate_move_token.len() <= 0 || candidate_move_token == game_result || candidate_move_token == "..." { continue; };

            // Comments continue to end of the line
            if candidate_move_token.starts_with(";") { return; }

            // Track start of a commentary block
            if candidate_move_token.starts_with("{") {
                commentary_found = true;
            }

            // Ensure to catch the end of a commentary block before skipping this token (if we're
            // already within a block)
            if candidate_move_token.ends_with("}") {
                commentary_found = false;
                continue;
            }

            // Skip any commentary blocks
            if commentary_found { continue; }

            // Skip move numbers
            if GAME_MOVE_NUMBER.find(candidate_move_token).is_some() { continue; }

            // If we made it here, we should finally be looking at a valid move token
            game_moves.push_back(String::from(candidate_move_token.trim()));
        }
    }

    fn get_next_pgn_game(&mut self) -> Option<(Position, VecDeque<String>, bool, bool)> {
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

                } else if GAME_MOVE_NUMBER.find(line.as_str()).is_some() || game_moves_found {
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
                return Some((position, game_moves, white_min_elo, black_min_elo));
            }
        }
    }

    // Returns a regex based on the PGN game move (in SAN notation) since SAN often omits
    // certain bits of information that are required to identify the current game move
    // unambiguously
    fn pgn_game_move_to_extended_san_regex(game_move: &str) -> Option<Regex> {
        // Castling moves should pass through unmodified
        if game_move.starts_with("O-") { return Some(Regex::new(format!("^{}$",  game_move).as_str()).unwrap()); };

        let mut match_re = Vec::<String>::with_capacity(32);
        let caps = GAME_MOVE_EXTENDED_SAN.captures(game_move)?;

        match_re.push(String::from("^"));
        match_re.push(String::from(caps.get(1).map(|m| if m.as_str().is_empty() {"P"} else {m.as_str()}).unwrap()));
        match_re.push(String::from(caps.get(2).map(|m| if m.as_str().is_empty() {"[a-h]"} else {m.as_str()}).unwrap()));
        match_re.push(String::from(caps.get(3).map(|m| if m.as_str().is_empty() {"[1-8]"} else {m.as_str()}).unwrap()));
        match_re.push(String::from(caps.get(4).unwrap().as_str()));
        match_re.push(String::from(caps.get(5).unwrap().as_str()));
        match_re.push(String::from(caps.get(6).map_or("", |m| m.as_str())));

        Some(Regex::new(&match_re.join("")).unwrap())
    }

    pub fn get_next_position(&mut self) -> Option<bool> {
        let mut new_game_loaded = false;

        loop {
            // Need to load a new game if there are 1 or fewer moves remaining
            // The final move is not played since it does not have a next move (target move) to train on
            if self.pgn_game_moves.len() <= 1 {
                let (pgn_game_position, pgn_game_moves, white_min_elo, black_min_elo) = self.get_next_pgn_game()?;

                self.pgn_game_position = pgn_game_position;
                self.pgn_game_moves = pgn_game_moves;
                self.white_min_elo = white_min_elo;
                self.black_min_elo = black_min_elo;

                new_game_loaded = true;

            } else {
                // If we already have a position loaded and a valid next move, then make it!
                // (i.e. apply it to the current position)
                let next_move_san = self.pgn_game_moves.pop_front().unwrap();

                // TODO: create a simple string comparison routine instead of using a Regex since they are super slow
                // Generate a new Regex from the PGN SAN move notation, since it doesn't fully specify all info required
                // to uniquely identify each move
                let next_move_extended_san_re = PGNReader::pgn_game_move_to_extended_san_regex(next_move_san.as_str());
                if next_move_extended_san_re.is_none() { continue; }

                // Locate the game move from all the possible ones
                let game_move = self.position_moves.get_move_by_extended_san(next_move_extended_san_re.unwrap()).expect(format!("Can't find source move {}", next_move_san.as_str()).as_str());

                // Make the move now
                self.move_maker.make_move(&mut self.pgn_game_position, &game_move, false);
            }

            // Update list of available moves in the position
            self.position_moves.clear();
            PositionAnalyzer::calc_legal_moves(&mut self.pgn_game_position, &mut self.position_moves);

            // Return an encoded position for the neural network only if the current player has reached the min ELO cutoff
            if (self.pgn_game_position.white_to_move && self.white_min_elo) || (!self.pgn_game_position.white_to_move && self.black_min_elo) {
                // TODO: return encoded position, encoded moves output mask, expected / target output


                return Some(new_game_loaded);

                // return Some(String::from("END"));
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
    }

    #[test]
    fn test_pgn_game_move_to_extended_san_regex() {
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("e4").unwrap().as_str(), "^P[a-h][1-8]e4");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("Nf3").unwrap().as_str(), "^N[a-h][1-8]f3");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("O-O").unwrap().as_str(), "^O-O$");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("O-O-O").unwrap().as_str(), "^O-O-O$");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("Bxc5").unwrap().as_str(), "^B[a-h][1-8]xc5");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("Nbd7").unwrap().as_str(), "^Nb[1-8]d7");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("cxb5").unwrap().as_str(), "^Pc[1-8]xb5");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("Nxe4").unwrap().as_str(), "^N[a-h][1-8]xe4");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("Bxf7+").unwrap().as_str(), "^B[a-h][1-8]xf7");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("R1a4#").unwrap().as_str(), "^R[a-h]1a4");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("Qh5xh1").unwrap().as_str(), "^Qh5xh1");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("f8=Q").unwrap().as_str(), "^P[a-h][1-8]f8=Q");
        assert_eq!(PGNReader::pgn_game_move_to_extended_san_regex("gxf8=B").unwrap().as_str(), "^Pg[1-8]xf8=B");
    }

    #[test]
    fn test_read_pgn_test_file() {
        // let file_path = "src/test/resources/bundesliga2000.pgn";
        let file_path = "src/test/resources/test_pgn1.pgn";
        let mut pgn = PGNReader::init_pgn_file(file_path);
        let mut total_games = 0;
        let mut total_positions = 0;
        println!("Initializing from file: {}", file_path);

        let before = Instant::now();

        loop {
            let x = pgn.get_next_position();
            if x.is_none() { break; }

            if total_positions % 1000 == 0 {
                println!("Total games: {}", total_games);
                println!("Total positions: {}", total_positions);
                let elapsed = before.elapsed();
                println!("Elapsed time: {:.2?}  ({:.1?} pos/s)", elapsed, (total_positions as f64 / elapsed.as_millis() as f64) * 1000f64);
                std::io::stdout().flush().unwrap_or(());
            }

            total_positions += 1;
            if x.unwrap() {total_games += 1; };
        }

        println!("Total games: {}", total_games);
        println!("Total positions: {}", total_positions);
        let elapsed = before.elapsed();
        println!("Elapsed time: {:.2?}  ({:.1?} pos/s)", elapsed, (total_positions as f64 / elapsed.as_millis() as f64) * 1000f64);
    }
}

