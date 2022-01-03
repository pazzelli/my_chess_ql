use regex::Regex;
use std::path::PathBuf;
use std::process::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::time::Instant;
use json::JsonValue;
use simple_error::SimpleError;
use crate::constants::PlayerColour;
use crate::game::analysis::kingattackrayanalyzer::KingAttackRayAnalyzer;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::moves::movemaker::MoveMaker;
use crate::game::position::Position;
use crate::game::positionhelper::PositionHelper;
use crate::test::movemakertesthelper::MoveMakerTestHelper;

pub struct StockfishInterface {}

impl StockfishInterface {
    pub fn open_stockfish() -> Option<Child> {
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

    pub fn write_to_stockfish(stockfish_in: &mut ChildStdin, commands: &str) {
        // println!("{}", commands);
        stockfish_in.write(commands.as_bytes()).unwrap();
        // stockfish_in.flush();
    }

    pub fn read_from_stockfish(stockfish_out: &mut BufReader<&mut ChildStdout>) -> (String, usize) {
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