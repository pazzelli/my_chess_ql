#[path = "../engine/EngineController.rs"] mod engine;

use std::io::{Write};
use engine::EngineController;

const HELLO_STRING: &str = "id name MyChessQL";
const AUTHOR_STRING: &str = "id author John Pazzelli";

pub struct UCIInterface { //<'a> {
    engine: EngineController
}

impl UCIInterface {
    pub fn init_interface() -> UCIInterface {
        UCIInterface {
            engine: engine::EngineController::init_engine()
        }
    }

    pub fn process_command(&mut self, cmd: &str) -> bool {
        let cmd_tokens: Vec<&str> = cmd.split_whitespace().collect();

        match cmd_tokens[0] {
            "uci" => {
                UCIInterface::send_to_gui(HELLO_STRING);
                UCIInterface::send_to_gui(AUTHOR_STRING);
                // println!("option name Hash type spin default 1 min 1 max 128");
                UCIInterface::send_to_gui("uciok");
            },

            "isready" => UCIInterface::send_to_gui("readyok"),

            "stop" => self.engine.stop_search(),

            "ucinewgame" => (),

            "position" => {
                match cmd_tokens[1] {
                    "fen" => self.engine.init_position(Some(&cmd_tokens[2..].join(""))),

                    "startpos" | _ => self.engine.init_position(None)   // start position is the default
                };

                // eng.position
                // //"fen rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1" => (),
            }

            "go" => {
                // movetime 3000"
                // UCIInterface::send_to_gui("info currmove d2d4 d7d5 currmovenumber 1");
                self.engine.start_search();
                UCIInterface::send_to_gui("info score cp 153  depth 1 nodes 13 time 15 pv d2d4 d7d5");
                UCIInterface::send_to_gui("bestmove d2d4");
            },

            "quit" => return false,

            // _ => println!("{} unknown", buffer),
            _ => (),
        }

        true
    }

    pub fn send_to_gui(msg: &str) {
        let mut msg = String::from(msg);
        if !msg.ends_with("\n") {msg.push_str("\n")};

        std::io::stdout()
            .write_all(msg.as_bytes())
            .expect("Failed to send message to ui");
        // Ok(())

        // io::stdout().write_all(b"hello world")?;
        //
        // Ok(())
    }
}
