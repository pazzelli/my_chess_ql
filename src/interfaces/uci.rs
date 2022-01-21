use crate::engine::enginecontroller::*;
use std::io::{Write};
use std::path::PathBuf;

const HELLO_STRING: &str = "id name MyChessQL";
const AUTHOR_STRING: &str = "id author John Pazzelli";

pub struct UCIInterface { //<'a> {
    engine: EngineController
}

impl UCIInterface {
    pub fn init_interface(nn_model_path: PathBuf) -> UCIInterface {
        UCIInterface {
            engine: EngineController::init(nn_model_path)
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

            "ucinewgame" => self.engine.init_new_game(),

            // TODO: need to support the 'moves' command within 'position' to play the list of moves on the board
            // from the position given in the FEN string:
            //   * position [fen <fenstring> | startpos ]  moves <move1> .... <movei>
            "position" => {
                match cmd_tokens[1] {
                    // "fen" => self.engine.init_position(Some(&cmd_tokens[2..].join(""))),
                    "fen" => self.engine.init_position(Some(&cmd_tokens[2..].join(" "))),

                    "startpos" | _ => self.engine.init_position(None)   // start position is the default
                };
            }

            "go" => {
                // movetime 3000 --> might be included in the 'go' command, will need to consider this later

                // Example of how to send the current move being considered back to the UI program
                // UCIInterface::send_to_gui("info currmove d2d4 d7d5 currmovenumber 1");

                // Get the best moves based on the NN prediction
                let (best_moves, _win_prob) = self.engine.get_best_moves();
                // UCIInterface::send_to_gui(format!("info score cp {:.7?}", win_prob).as_str());

                // Need to flip the evaluation in the UI so it doesn't always show negative numbers for black
                // For now, the evaluation shows the probability of making the move, not the actual position strength
                let multiplier = (((self.engine.position.as_ref().unwrap().white_to_move as i8) << 1) - 1) as f32;
                for i in 0..best_moves.len() {
                    // In case there are fewer than K available moves
                    if best_moves[i].0.is_none() { break; }

                    // Send info about all K top moves to the UI using the 'info multipv' command
                    // the 'time 1' is necessary here so ChessX doesn't ignore the line entirely
                    let game_move = best_moves[i].0.unwrap();
                    UCIInterface::send_to_gui(format!("info multipv {} score cp {:.0?} depth 1 nodes 1 time 1 pv {}", (i+1), (best_moves[i].1) * multiplier * 100f32, game_move.get_uci_move_string()).as_str());

                    // Sample command:
                    // UCIInterface::send_to_gui("info score cp 153  depth 1 nodes 13 time 15 pv d2d4 d7d5");
                }
                // Sleep 1s to allow the info lines above to be displayed in the UI for long enough to see them
                std::thread::sleep(std::time::Duration::from_millis(1000));

                // For now, just send the top move but this can be adjusted later
                UCIInterface::send_to_gui(format!("bestmove {:?}", best_moves[0].0.unwrap()).as_str());
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
