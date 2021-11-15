use crate::constants::*;
use crate::game::position::*;
use crate::game::gamemovelist::*;

pub trait Piece {
    fn calc_attacked_squares(position: &Position, player: PlayerColour) -> u64;

    fn calc_movements(position: &Position, move_list: &mut GameMoveList, enemy_attacked_squares: Option<u64>) -> (u64, u64);
}