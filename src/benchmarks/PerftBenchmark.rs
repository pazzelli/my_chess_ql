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
use crate::interfaces::stockfish::StockfishInterface;
use crate::test::movemakertesthelper::MoveMakerTestHelper;

pub struct PerftBenchmark {}

impl PerftBenchmark {
    fn run_stockfish_analysis(fen: &str, stockfish: &mut Child) -> String {
        let mut stockfish_in = stockfish.stdin.as_mut().unwrap();
        let mut stockfish_out = BufReader::new(stockfish.stdout.as_mut().unwrap());

        let commands = format!("position fen {}\ngo perft 1\n", fen);

        StockfishInterface::write_to_stockfish(&mut stockfish_in, commands.as_str());
        let (moves, _move_count) = StockfishInterface::read_from_stockfish(&mut stockfish_out);
        moves
    }

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

    pub fn run_perft_recursive(position: &mut Position, depth: u8, intense_verify: bool, stockfish: &mut Option<Child>) -> Result<usize, SimpleError> {
        // if depth <= 0 { return Ok(1); }

        let mut move_maker = MoveMaker::default();
        let mut move_list = GameMoveList::default();
        PositionAnalyzer::calc_legal_moves(position, &mut move_list);

        if intense_verify {
            let fen = position.to_fen();
            let mut temp_pos = Position::from_fen(Some(fen.as_str()), false).unwrap();
            let mut temp_move_list = GameMoveList::default();
            PositionAnalyzer::calc_legal_moves(&mut temp_pos, &mut temp_move_list);

            // Ensure move lists from the current / reused Position object match with that generated from a fresh one
            PerftBenchmark::compare_move_lists(&position, &move_list, &temp_pos, &temp_move_list)?;

            if stockfish.is_some() {
                // let (sf_moves, sf_move_count) = run_stockfish_analysis(fen.as_str(), stockfish.as_mut().unwrap());
                let sf_moves = PerftBenchmark::run_stockfish_analysis(fen.as_str(), stockfish.as_mut().unwrap());
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

        if depth <= 1 {
            return Ok(move_list.list_len);
        }

        let mut nodes = 0;
        for i in 0..move_list.list_len {
            move_maker.make_move(position, &move_list.move_list[i], true);

            match PerftBenchmark::run_perft_recursive(position, depth - 1, intense_verify, stockfish) {
                Ok(n) => nodes += n,
                Err(e) => {
                    println!("Last move played: {:?}", move_list.move_list[i]);
                    return Err(e);
                }
            }

            move_maker.unmake_move(position, &move_list.move_list[i]);
        }
        Ok(nodes)
    }

    // see: run_legal_moves_test_cases() -> this needs to be refactored
    pub fn run_perft(fen_str: Option<&str>, max_depth: u8, debug: bool) {
        // Tests just a basic perft run from the position + depth specified
        // Starting position
        let mut stockfish = if debug { StockfishInterface::open_stockfish() } else { None };

        println!("\nStarting perft (move generator) benchmark to depth {}...", max_depth);
        println!("\nStart position:");
        for depth in 1..max_depth+1 {
            let mut position = Position::from_fen(fen_str, depth == 1).unwrap();

            let before = Instant::now();
            match PerftBenchmark::run_perft_recursive(&mut position, depth, debug, &mut stockfish) {
                Ok(result) => {
                    let elapsed = before.elapsed();
                    println!("Depth: {}\tNodes: {:}\t\tElapsed: {:.2?}  ({:.1?} pos/s)", depth, result, elapsed, (result as f64 / elapsed.as_millis() as f64) * 1000f64);
                },
                Err(_) => ()
            };

        }

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
}