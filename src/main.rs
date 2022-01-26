
// #![feature(portable_simd)]
#![allow(dead_code, unused_imports)]
#[macro_use] extern crate lazy_static;
extern crate regex;
extern crate clap;

#[cfg(not(compile_training))]
extern crate tensorflow;

mod constants;
mod engine;
mod game;
mod interfaces;
mod neural;
mod benchmarks;
mod test;
use std::path::PathBuf;

use std::{env, io};
// use std::io::Write;
use std::time::Instant;
use simple_error::SimpleError;
use interfaces::*;
use constants::*;
use clap::{Arg, App, SubCommand, ArgGroup};
use game::pieces::piece::*;
use game::analysis::positionanalyzer::*;
use game::position::*;
use game::positionhelper::*;
use game::moves::gamemovelist::*;
use crate::interfaces::pgn::PGNReader;
use crate::benchmarks::perftbenchmark::PerftBenchmark;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::movemaker::MoveMaker;
use crate::game::pieces::king::King;
use crate::test::legalmoveshelper::LegalMovesTestHelper;
use crate::test::movemakertesthelper::MoveMakerTestHelper;

/// Processes incoming commands from stdin indefinitely
fn process_ui_commands(uci_interface: &mut uci::UCIInterface) {
    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(_n_bytes) => {
                let cmd = buffer.as_str().trim();
                if !uci_interface.process_command(&cmd) { break; }

                // println!("{} bytes read", n);
                // println!("{}", buffer);
            }
            Err(e) => println!("Error while reading from stdin: {}", e),
        }
    }
}

/// Returns the full path to the current TF model directory based on a relative input path
fn get_nn_model_dir(model_dir: &str) -> PathBuf {
    #[cfg(debug_assertions)]
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // For release builds, assume the model is beneath the EXE's current folder
    #[cfg(not(debug_assertions))]
    let mut path = std::env::current_exe().unwrap().parent().unwrap().to_path_buf();

    println!("{}", path.display());

    path.push(model_dir);
    path
}

fn main() {
    let matches = App::new("MyChessQL Chess Engine")
        .author("John Pazzelli <john.pazzelli@gmail.com>")
        .long_about("Neural Network-based chess engine written in Rust\nIf no options specified, listens on stdin for UCI commands (i.e. for use when connected to a chess UI application)")
        .subcommand(SubCommand::with_name("perft")
            .about("runs the perft (move generator) benchmark test")
            // .group(ArgGroup::with_name("vers")
            //     .args(&["pgn", "perft"])
            //     .required(true))
            .arg(Arg::with_name("depth")
             .long("depth")
             .value_name("DEPTH")
             .default_value("3"))
            .arg(Arg::with_name("fen")
             .long("fen")
             .value_name("FEN_STR"))
            .arg(Arg::with_name("debug")
             .long("debug")
             .help("perform 'intense verification' (uses stockfish, if installed - reduces performance but finds bugs)"))
            )
        .get_matches();

    // Run perft benchmark, if specified
    if let Some(matches) = matches.subcommand_matches("perft") {
        let mut depth = 4;
        let mut fen: Option<&str> = None;
        if matches.is_present("depth") {
            depth = matches.value_of("depth").unwrap().parse().unwrap();
        }
        if matches.is_present("fen") {
            fen = matches.value_of("fen");
        }
        return PerftBenchmark::run_perft(fen, depth, matches.is_present("debug"));
    }

    // Default invocation - wait for input command line args from a chess UI program
    let mut uci = uci::UCIInterface::init_interface(get_nn_model_dir("models"));
    process_ui_commands(&mut uci);
}
