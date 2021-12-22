
// #![feature(portable_simd)]
#![allow(dead_code, unused_imports)]
#[macro_use] extern crate lazy_static;
extern crate regex;
mod constants;
mod engine;
mod game;
mod interfaces;
// mod interfaces_python;
// mod neural;
mod test;

use std::path::PathBuf;
use std::{io};
// use std::io::Write;
use std::time::Instant;
use simple_error::SimpleError;
use interfaces::*;
use constants::*;
use game::pieces::piece::*;
use game::analysis::positionanalyzer::*;
use game::position::*;
use game::positionhelper::*;
use game::moves::gamemovelist::*;
// use crate::interfaces_python::pgn::PGNReader;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::movemaker::MoveMaker;
use crate::game::pieces::king::King;
use crate::test::legalmoveshelper::LegalMovesTestHelper;
use crate::test::movemakertesthelper::MoveMakerTestHelper;


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

fn main() {
    let mut uci = uci::UCIInterface::init_interface();
    process_ui_commands(&mut uci);

    // // Useful to have this here for profiling the binary externally using flamegraph
    // let iterations = 10000000;   // currently about 8.5s after calculating and storing pawn moves only
    // // let iterations = 100;
    //
    // let mut position = game::position::Position::from_fen(Some("r2q1rk1/pP2ppbp/2p2np1/PpPPP1B1/51b1/Q4N1P/5PP1/3RKB1R w KQkq b6 1 2")).unwrap();
    // let mut move_list = game::gamemovelist::GameMoveList::default();
    // let before = Instant::now();
    // for _ in 0..iterations {
    //     move_list.clear();
    //     game::positionanalyzer::PositionAnalyzer::calc_legal_moves(&mut position, &mut move_list);
    //     // println!("{:?}", MoveGenerator::calc_legal_moves(&position).move_list);
    // }
    // println!("Elapsed time: {:.2?}", before.elapsed());
    // // println!("{:?}", move_list);
}
