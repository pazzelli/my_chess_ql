use rand::prelude::*;
use crate::game::analysis::positionanalyzer::PositionAnalyzer;
use crate::game::moves::gamemovelist::GameMoveList;
use crate::game::position::*;

pub struct EngineController {
    pub position: Option<Position>,
}

impl Default for EngineController {
    fn default() -> Self {
        EngineController {
            position: None
        }
    }
}

impl EngineController {
    pub fn init_position(&mut self, fen_str: Option<&str>) {
        self.position = Some(Position::from_fen(fen_str, false).expect("Invalid FEN string"));
    }

    pub fn get_best_move(&mut self) -> String {
        let mut move_list = GameMoveList::default();
        PositionAnalyzer::calc_legal_moves(&mut self.position.as_mut().unwrap(), &mut move_list);

        let random_move_index = (rand::random::<f32>() * move_list.list_len as f32).floor() as usize;
        format!("{:?}", move_list.move_list[random_move_index])
    }

    pub fn stop_search(&self) {

    }
}