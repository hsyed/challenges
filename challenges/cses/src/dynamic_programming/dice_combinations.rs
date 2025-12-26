use std::collections::VecDeque;

//https://cses.fi/problemset/task/1633
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let n: usize = input.parse();

    let result = count_ways(n);

    out.println(result);
}

// Count the number of ways to construct sum n by throwing a dice one or more times
// Each throw produces a result between 1 and 6
// Answer modulo 10^9 + 7
fn count_ways(n: usize) -> usize {
    let mut tally = VecDeque::from([1, 1, 2, 4, 8, 16]);
    if n <= 6 {
        tally.iter().take(n).sum::<usize>() % 1_000_000_007
    } else {
        for _ in 0..n - 6 {
            let next = tally.iter().sum::<usize>() % 1_000_000_007;
            tally.push_back(next);
            tally.pop_front();
        }
        tally.iter().sum::<usize>() % 1_000_000_007
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_under_6() {
        // Example: n=3 should have 4 ways: [1,1,1], [1,2], [2,1], [3]
        assert_eq!(count_ways(3), 4);
        assert_eq!(count_ways(6), 32);
    }
}
