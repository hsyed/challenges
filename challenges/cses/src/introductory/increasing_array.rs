//https://cses.fi/problemset/task/1094
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let n: usize = input.parse();
    let nums: Vec<u64> = input.parse_vec();
    out.println(do_solve(&nums));
}

fn do_solve(nums: &[u64]) -> u64 {
    let mut increments = 0;
    let mut prev = nums[0];

    for &num in &nums[1..] {
        if num < prev {
            increments += prev - num;
        } else {
            prev = num;
        }
    }

    increments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(5, do_solve(&[3, 2, 5, 1, 7]));
    }
}
