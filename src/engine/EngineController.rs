use crate::game::position::*;

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