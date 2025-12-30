// Dynamic Programming problems module
// Add problem modules here as they are implemented

pub mod array_description;
pub mod book_shop;
pub mod coin_combinations_1;
pub mod dice_combinations;
pub mod minimizing_coins;
pub mod removing_digits;

use crate::TaskGroup;

pub fn tasks() -> TaskGroup {
    TaskGroup::new("dynamic_programming")
        .add("array_description", array_description::solve)
        .add("book_shop", book_shop::solve)
        .add("coin_combinations_1", coin_combinations_1::solve)
        .add("dice_combinations", dice_combinations::solve)
        .add("minimizing_coins", minimizing_coins::solve)
        .add("removing_digits", removing_digits::solve)
}
