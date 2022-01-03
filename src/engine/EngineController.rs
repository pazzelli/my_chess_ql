use rand::prelude::*;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::position::*;
use crate::neural::positionconverter::PositionConverter;

pub struct EngineController {
    pub position: Option<Position>,
    pub nn_converter: PositionConverter
}

impl Default for EngineController {
    fn default() -> Self {
        EngineController {
            position: None,
            nn_converter: PositionConverter::new()
        }
    }
}

impl EngineController {
    pub fn init_new_game(&mut self) {
        self.nn_converter.init_new_game();
    }

    pub fn init_position(&mut self, fen_str: Option<&str>) {
        self.position = Some(Position::from_fen(fen_str, false).expect("Invalid FEN string"));
    }

    pub fn get_best_move(&mut self) -> String {
        let mut move_list = GameMoveList::default();
        PositionAnalyzer::calc_legal_moves(&mut self.position.as_mut().unwrap(), &mut move_list);

        // TODO: add neural net position conversion logic
        // let (input_data, output_mask) = self.nn_converter.convert_position_for_nn(&self.pgn_game_position, &self.position_moves);

        // TODO: save random move logic as an engine option
        let random_move_index = (rand::random::<f32>() * move_list.list_len as f32).floor() as usize;
        format!("{:?}", move_list.move_list[random_move_index])
    }

    pub fn stop_search(&self) {

    }
}