#[path = "../constants.rs"] mod constants;
#[path = "../game/GameMove.rs"] mod game_move;
#[path = "../game/GameMoveList.rs"] mod game_move_list;
#[path = "../game/Position.rs"] mod position;
#[path = "../game/PositionHelper.rs"] mod position_helper;
#[path = "../game/MoveGenerator.rs"] mod position_analyzer;

use position::Position;
// use position_analyzer::PositionAnalyzer;

pub struct EngineController {
    // pub(crate) position: position::Position, // Temporary until MinimaxSearcher is implemented
}

impl EngineController {
    pub fn init_engine() -> EngineController {
        EngineController {
            // position: position::Position {}
        }
    }

    pub fn init_position(&mut self, fen_str: Option<&str>) -> Option<Position> {
        let position = Position::from_fen(fen_str).unwrap();

        // PositionAnalyzer.
        Some(position)

    }

    pub fn start_search(&self) {

    }

    pub fn stop_search(&self) {

    }
}