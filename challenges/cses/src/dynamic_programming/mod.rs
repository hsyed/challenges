// Dynamic Programming problems module
// Add problem modules here as they are implemented

pub mod dice_combinations;

use crate::TaskGroup;

pub fn tasks() -> TaskGroup {
    TaskGroup::new("dynamic_programming").add("dice_combinations", dice_combinations::solve)
}
