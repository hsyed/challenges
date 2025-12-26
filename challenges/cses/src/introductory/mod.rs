// Introductory problems module
// Add problem modules here as they are implemented
//
// Example:
// pub mod weird_algorithm;
// pub mod missing_number;
// pub mod repetitions;

pub mod digit_queries;
pub mod grid_path_description;

use crate::TaskGroup;

pub fn tasks() -> TaskGroup {
    TaskGroup::new("introductory")
        .add("digit_queries", digit_queries::solve)
        .add("grid_path_description", grid_path_description::solve)
}
