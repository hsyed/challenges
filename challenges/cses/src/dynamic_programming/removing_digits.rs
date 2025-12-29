use std::collections::VecDeque;

//https://cses.fi/problemset/task/1637
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let n: usize = input.parse();

    let result = min_steps(n);

    out.println(result);
}

struct DigiIterator {
    n: usize,
    divisor: usize,
}

impl DigiIterator {
    fn from(n: usize) -> Self {
        if n == 0 {
            return Self { n: 0, divisor: 1 };
        }

        // Find the highest divisor (e.g., 1000 for 1234)
        let mut divisor = 1;
        let mut temp = n;
        while temp >= 10 {
            temp /= 10;
            divisor *= 10;
        }

        Self { n, divisor }
    }
}

impl Iterator for DigiIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.divisor == 0 {
            return None;
        }

        let digit = self.n / self.divisor;
        self.n %= self.divisor;
        self.divisor /= 10;

        Some(digit)
    }
}

// Calculate minimum steps to reduce n to 0 by subtracting digits
fn min_steps(n: usize) -> usize {
    if n == 0 {
        return 0;
    } else if n < 10 {
        return 1;
    }

    let mut tally = VecDeque::from([1usize; 9]);

    for i in 10..=n {
        let selected = DigiIterator::from(i)
            .filter(|&d| d > 0)
            .map(|d| tally[9 - d] + 1)
            .min()
            .unwrap();
        tally.pop_front();
        tally.push_back(selected);
    }

    tally.back().copied().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Example: n=27 should take 5 steps: 27->20->18->10->9->0
        assert_eq!(min_steps(27), 5);
    }
}

