// Dynamic Programming problems module
// Add problem modules here as they are implemented

pub mod dice_combinations;
pub mod minimizing_coins;

use crate::TaskGroup;

pub fn tasks() -> TaskGroup {
    TaskGroup::new("dynamic_programming")
        .add("dice_combinations", dice_combinations::solve)
        .add("minimizing_coins", minimizing_coins::solve)
}
