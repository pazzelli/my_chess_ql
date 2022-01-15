use std::path::PathBuf;
use rand::prelude::*;
use crate::constants::*;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemove::GameMove;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::position::*;
use crate::neural::nnprediction::NNPrediction;
use crate::neural::positionconverter::NNPositionConverter;

pub struct EngineController {
    pub position: Option<Position>,
    pub nn_predictor: NNPrediction,
}

impl EngineController {
    pub fn init(nn_model_dir: PathBuf) -> EngineController {
        EngineController {
            position: None,
            nn_predictor: NNPrediction::init_from_saved_model(nn_model_dir).unwrap(),
        }
    }

    pub fn init_new_game(&mut self) {
        self.nn_predictor.init_new_game();
    }

    pub fn init_position(&mut self, fen_str: Option<&str>) {
        self.position = Some(Position::from_fen(fen_str, false).expect("Invalid FEN string"));
    }

    /// Returns the top K best moves for teh given input position
    pub fn get_best_moves(&mut self) -> ([(Option<GameMove>, f32); TOP_K_OUTPUTS], f32) {
        let mut pos = self.position.as_mut().unwrap();
        self.nn_predictor.make_prediction(&mut pos)

        // // TODO: save random move logic as an engine option
        // let mut move_list = GameMoveList::default();
        // PositionAnalyzer::calc_legal_moves(&mut self.position.as_mut().unwrap(), &mut move_list);
        // let random_move_index = (rand::random::<f32>() * move_list.list_len as f32).floor() as usize;
        // format!("{:?}", move_list.move_list[random_move_index])
    }

    pub fn stop_search(&self) {

    }
}