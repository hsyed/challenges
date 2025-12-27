// Dynamic Programming problems module
// Add problem modules here as they are implemented

pub mod coin_combinations_1;
pub mod dice_combinations;
pub mod minimizing_coins;

use crate::TaskGroup;

pub fn tasks() -> TaskGroup {
    TaskGroup::new("dynamic_programming")
        .add("coin_combinations_1", coin_combinations_1::solve)
        .add("dice_combinations", dice_combinations::solve)
        .add("minimizing_coins", minimizing_coins::solve)
}
