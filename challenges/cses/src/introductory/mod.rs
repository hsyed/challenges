// Introductory problems module
// Add problem modules here as they are implemented
//
// Example:
// pub mod weird_algorithm;
// pub mod missing_number;
// pub mod repetitions;

pub mod digit_queries;
pub mod grid_path_description;
pub mod increasing_array;
pub mod missing_number;
pub mod repetitions;
pub mod string_reorder;
pub mod weird_algorithm;

use crate::TaskGroup;

pub fn tasks() -> TaskGroup {
    TaskGroup::new("introductory")
        .add("digit_queries", digit_queries::solve)
        .add("grid_path_description", grid_path_description::solve)
        .add("increasing_array", increasing_array::solve)
        .add("missing_number", missing_number::solve)
        .add("repetitions", repetitions::solve)
        .add("string_reorder", string_reorder::solve)
        .add("weird_algorithm", weird_algorithm::solve)
}
