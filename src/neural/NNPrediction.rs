use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(not(compile_training))]
use tensorflow::{Graph, SavedModelBundle, SessionOptions, SessionRunArgs, Tensor, Status};

use crate::constants::*;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::position::Position;
use crate::neural::positionconverter::NNPositionConverter;

// Create a dummy implementation of the NNPrediction class when compiling for training
// This essentially eliminates the rust-tensorflow dependency at the code level
#[cfg(compile_training)]
pub struct NNPrediction {
    // Dummy
}

#[cfg(compile_training)]
impl NNPrediction {
    pub fn init_from_saved_model(_model_dir: PathBuf) -> Result<NNPrediction, Status> {
        Ok(NNPrediction{})
    }

    pub fn init_new_game(&mut self) { }

    pub fn make_prediction(&mut self, _position: &mut Position) -> ([(Option<GameMove>, f32); TOP_K_OUTPUTS], f32) {
        ([(None, 0f32); TOP_K_OUTPUTS], 0f32)
    }
}

#[cfg(not(compile_training))]
pub struct NNPrediction {
    graph: Graph,
    saved_model: SavedModelBundle,
    nn_converter: NNPositionConverter,
    // predictor: Py<PyAny>
}

#[cfg(not(compile_training))]
impl NNPrediction {
    /// Opens the specified saved model, import the graph and create the TF Session
    pub fn init_from_saved_model(model_dir: PathBuf) -> Result<NNPrediction, Status> {
        if !model_dir.exists() {
            panic!("TensorFlow model not found: {}", model_dir.display());
        }

        // Load the saved model exported by regression_savedmodel.py.
        let mut graph = Graph::new();
        let bundle =
            SavedModelBundle::load(&SessionOptions::new(), &["serve"], &mut graph, model_dir)?;

        Ok(NNPrediction {
            graph,
            saved_model: bundle,
            nn_converter: NNPositionConverter::new(),
        })
    }

    pub fn init_new_game(&mut self) {
        self.nn_converter.init_new_game();
    }

    /// Makes a prediction through the neural network and returns the top K moves
    pub fn make_prediction(&mut self, position: &mut Position) -> ([(Option<GameMove>, f32); TOP_K_OUTPUTS], f32) {
        // Calc all legal moves in the position - required to mask out the output vector to only
        // consider legal moves (done prior to applying the final softmax activation to the outputs)
        let mut move_list = GameMoveList::default();
        PositionAnalyzer::calc_legal_moves(position, &mut move_list);

        // Make the prediction
        let pred = self.make_prediction_from_nn(&position, &move_list).unwrap();

        // Locate and return the top K gave moves and discard the rest
        let mut top_k_moves: [(Option<GameMove>, f32); TOP_K_OUTPUTS] = [(None, 0f32); TOP_K_OUTPUTS];
        // pred.1 is the top move count returned by the NN
        let max_ind = usize::min(usize::min(TOP_K_OUTPUTS, move_list.list_len), pred.1);
        for i in 0..max_ind {
            // println!("{}, {}", pred.0[i].0, pred.0[i].1);
            top_k_moves[i] = (Some(move_list.move_list[pred.0[i].0 as usize]), pred.0[i].1)
        }

        (top_k_moves, pred.2)
    }

    /// Converts the position and game moves into the necessary inputs for the NN, then invokes
    /// the TensorFlow graph to produce the actual outputs
    fn make_prediction_from_nn(&mut self, position: &Position, game_move_list: &GameMoveList) -> Result<([(i16, f32); TOP_K_OUTPUTS], usize, f32), Status> {
        // Sample usage / example, taken from:
        // https://github.com/tensorflow/rust/blob/master/examples/regression_savedmodel.rs

        // inputs:  [main_input, main_output_mask]
        let (input_data, output_mask) = self.nn_converter.convert_position_for_nn(&position, &game_move_list);
        let main_input = Tensor::new(&[1, NN_TOTAL_INPUT_SIZE_PER_POS as u64]).with_values(&input_data.as_slice())?;
        let main_output_mask = Tensor::new(&[1, NN_TOTAL_OUTPUT_SIZE_PER_POS as u64]).with_values(&output_mask.as_slice())?;

        // Load the 'serving_default' model call graph signature, which is added by default
        // by Keras when saving the model
        let call_signature = self.saved_model.meta_graph_def().get_signature("serving_default")?;
        // // How to debug the call signature (see inputs/outputs):
        // println!("sig name: {}", &call_signature.name().to_string());
        // println!("{:?}", call_signature.inputs());
        // println!("{:?}", call_signature.outputs());

        // Retrieve the inputs & outputs
        let main_input_info = call_signature.get_input(INPUT_MAIN_LAYER_NAME).unwrap();
        let output_mask_info = call_signature.get_input(OUTPUT_MASK_LAYER_NAME).unwrap();
        // let movement_output_info = call_signature.get_output(OUTPUT_MOVEMENTS_LAYER_NAME).unwrap();
        let top_k_output_info = call_signature.get_output(OUTPUT_TOP_K_MOVEMENTS_LAYER_NAME).unwrap();
        let win_probability_info = call_signature.get_output(OUTPUT_WIN_PROBABILITY_LAYER_NAME).unwrap();

        // Retrieve the graph operations required to bind the inputs / outputs
        let op_main_inp = self.graph.operation_by_name_required(&main_input_info.name().name)?;
        let op_output_mask = self.graph.operation_by_name_required(&output_mask_info.name().name)?;
        // let op_movement_output = self.graph.operation_by_name_required(&movement_output_info.name().name)?;
        let op_top_k_outputs = self.graph.operation_by_name_required(&top_k_output_info.name().name)?;
        let op_win_probability = self.graph.operation_by_name_required(&win_probability_info.name().name)?;

        // Create a session, required to run operations in the graph, and bind the inputs/outputs
        let mut call_step = SessionRunArgs::new();
        call_step.add_feed(&op_main_inp, 0, &main_input);
        call_step.add_feed(&op_output_mask, 0, &main_output_mask);
        // call_step.add_target(&op_movement_output);
        call_step.add_target(&op_top_k_outputs);
        call_step.add_target(&op_win_probability);

        // Grab the data out of the session using a fetch token
        // let movement_output_fetch = call_step.request_fetch(&op_movement_output, 0);
        let top_k_outputs_fetch = call_step.request_fetch(&op_top_k_outputs, 1);
        let win_probability_fetch = call_step.request_fetch(&op_win_probability, 2);

        // Run the session / graph operations
        self.saved_model.session.run(&mut call_step)?;

        // Retrieve outputs as tensors
        // let movement_output: Tensor<f32> = call_step.fetch(movement_output_fetch)?;
        let top_k_outputs: Tensor<f32> = call_step.fetch(top_k_outputs_fetch)?;
        let win_probability: f32 = call_step.fetch(win_probability_fetch)?[0];

        // println!("\nTop k outputs: {:?}", top_k_outputs.iter());
        // println!("\nwin probability: {:?}", win_probability);

        let (top_k_game_move_indices, top_moves_found) = NNPrediction::convert_nn_movements_to_game_move_indices(top_k_outputs, &game_move_list, !position.white_to_move);
        Ok((top_k_game_move_indices, top_moves_found, win_probability))
    }

    /// Converts the index of the top K neural net output neurons (in the top_k_movements tensor)
    /// into an array of indices into the GameMoveList (to locate the appropriate GameMove) for each one
    /// The &GameMove objects themselves cannot be returned since they would then have different lifetimes
    /// than the GameMoveList that is passed in here
    fn convert_nn_movements_to_game_move_indices(top_k_movements: Tensor<f32>, game_move_list: &GameMoveList, flip_for_black: bool) -> ([(i16, f32); TOP_K_OUTPUTS], usize) {
        let mut top_k_out = [(-1i16, 0f32); TOP_K_OUTPUTS];

        // Build a temporary map of source / target squares -> index in the top_k array
        // This is for efficiency so we can just loop over all the game moves one time and resolve
        // all top K movements in a single pass (below)
        let mut top_k_src_tgt_squares = HashMap::<(u8, u8), usize>::with_capacity(TOP_K_OUTPUTS);
        for i in 0..TOP_K_OUTPUTS {
            let (source_square, target_square, _promotion_piece) = NNPositionConverter::decode_movement(top_k_movements[i] as u16, flip_for_black);
            top_k_src_tgt_squares.insert((source_square, target_square), i);
            // println!("#{}: src square {}, tgt square {}", (i+1), source_square, target_square);
        }

        // Loop over game moves in a single pass
        let mut top_moves_found = 0usize;
        for i in 0..game_move_list.move_list.len() {
            // If this game move's source square is in the map created above, it is in the top K moves
            let move_source_sq = game_move_list.move_list[i].source_square;
            let move_target_sq = game_move_list.move_list[i].target_square;
            if top_k_src_tgt_squares.contains_key(&(move_source_sq, move_target_sq)) {
                // Use the temporary map to resolve the index in the top K array
                let top_k_array_ind = top_k_src_tgt_squares[&(move_source_sq, move_target_sq)];

                // The top_k_movements output by the NN has the softmax percentage / probability offset exactly K positions
                // away from the movement index, since this tensor is a concatenation of the movement index + probabilities
                top_k_out[top_k_array_ind] = (i as i16, top_k_movements[top_k_array_ind + TOP_K_OUTPUTS]);
                top_moves_found += 1;

                if top_moves_found >= TOP_K_OUTPUTS { break; }
            }
        }

        (top_k_out, top_moves_found)
    }

    // ANOTHER METHOD CALLING PYTHON DIRECTLY FROM RUST, BUT THIS WILL BE DIFFICULT TO DEPLOY
    // AND I NEVER GOT IT WORKING PROPERLY:

    // /// Opens the specified saved model, import the graph and create the TF Session
    // pub fn init_from_saved_model(model_dir: &str) -> Result<NNPrediction> {
    //     pyo3::prepare_freethreaded_python();
    //
    //     if !Path::new(model_dir).exists() {
    //         panic!("TensorFlow model not found: {}", model_dir);
    //     }
    //
    //     //Combine env::current_exe() with Path::join() and fs::canonicalize() to get absolute path relative to the executable. That's reasonable for data files distributed with the program, e.g. for a game.
    //
    //     let predictor: Py<PyAny> = Python::with_gil(| py| {
    //         let sys = PyModule::import(py, "sys").unwrap();
    //         let executable = sys.getattr("executable").unwrap().to_string();
    //         let path = sys.getattr("path").unwrap().to_string();
    //         println!("{}\n{}", executable.as_str(), path.as_str());
    //
    //         // // Prints:
    //         // /Users/John/IdeaProjects/MyChessQL/target/debug/my_chess_ql
    //         // ['/usr/local/Cellar/python@3.8/3.8.6/Frameworks/Python.framework/Versions/3.8/lib/python38.zip', '/usr/local/Cellar/python@3.8/3.8.6/Frameworks/Python.framework/Versions/3.8/lib/python3.8', '/usr/local/Cellar/python@3.8/3.8.6/Frameworks/Python.framework/Versions/3.8/lib/python3.8/lib-dynload', '/usr/local/Cellar/python@3.8/3.8.6/Frameworks/Python.framework/Versions/3.8/lib/python3.8/site-packages']
    //
    //
    //         let prediction_module = PyModule::import(py, "prediction").unwrap();
    //         let prediction_class = prediction_module.getattr("Prediction").unwrap();
    //         let prediction_instance = prediction_class.call1((model_dir, )).unwrap();
    //
    //         // let house_class = custom_manager.getattr("House").unwrap();
    //         // let house = house_class.call1(("123 Main Street",)).unwrap();
    //
    //         prediction_instance.into()
    //     });
    //
    //     // Load the saved model exported by regression_savedmodel.py.
    //     Ok(NNPrediction {
    //         nn_converter: PositionConverter::new(),
    //         predictor,
    //     })
    // }

    // pub fn make_prediction(&mut self, position: &Position, game_move_list: &GameMoveList) -> Result<()> {
    //     // let (input_data, output_mask) = self.nn_converter.convert_position_for_nn(&position, &game_move_list);
    //
    //     // // let (final_output, win_probability_output) = Python::with_gil(|py| {
    //     // let result = Python::with_gil(|py| {
    //     //      self.predictor.getattr(py,"predict").unwrap().call1(py, (input_data, output_mask)).unwrap()
    //     //
    //     //     // let total: i32 = builtins.getattr("sum")?.call1((vec![1, 2, 3],))?.extract()?;
    //     // });
    //     //
    //     // println!("{:?}", result);
    //     // Ok(())
    // }
}

#[cfg(test)]
mod tests {
    use arrayvec::ArrayString;
    use float_cmp::{approx_eq, assert_approx_eq};
    use std::collections::HashMap;
    use itertools::min;
    use simple_error::bail;
    use crate::game::analysis::positionanalyzer::PositionAnalyzer;
    use crate::game::moves::movemaker::MoveMaker;
    use crate::PGNReader;
    use super::*;

    #[test]
    fn test_make_prediction_start_position() {
        let mut position = Position::from_fen(None, false).unwrap();
        // // let mut position = Position::from_fen(Some("r2q1rk1/pp2ppbp/2p2np1/2pPP1B1/8/Q5np/P1P2PP1/3RKB1R w KQkq - 1 2"), false).unwrap();

        let mut predictor = NNPrediction::init_from_saved_model(PathBuf::from("/Users/John/IdeaProjects/MyChessQL/models")).unwrap();

        let (best_moves, win_prob) = predictor.make_prediction(&mut position);
        for i in 0..best_moves.len() {
            // Ensure that the game moves were properly found
            let mut game_move = best_moves[i].0.expect(format!("Top game move #{} not found", (i+1)).as_str());

            game_move.set_extended_san_move_string();
            println!("#{}: {}, {:.3?}", (i+1), game_move.extended_move_san, best_moves[i].1);
        }
        println!("\nWin probability: {:.7?}\n", win_prob);

        // // match predictor.make_prediction(&position, &game_move_list) {
        // match predictor.make_prediction(&mut position) {
        //     Ok((top_k_move_indices, win_prob)) => {
        //         println!("\nTop moves:");
        //         for i in 0..top_k_move_indices.len() {
        //             // // Ensure that the game move index was properly found
        //             // assert_ne!(top_k_move_indices[i].0, -1i16);
        //             //
        //             // let mut game_move = game_move_list.move_list[top_k_move_indices[i].0 as usize];
        //             // game_move.set_extended_san_move_string();
        //             // println!("#{}: {}, {:.3?}", (i+1), game_move.extended_move_san, top_k_move_indices[i].1);
        //         }
        //         println!("\nWin probability: {:.7?}\n", win_prob);
        //     }
        //     Err(e) => {
        //         panic!("Position prediction returned an error: {}", e);
        //     },
        // }
    }
}