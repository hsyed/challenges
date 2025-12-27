//https://cses.fi/problemset/task/1083
use crate::{Scanner, Writer};

// * The artimetic series sum (sum 1..n) formula is n * (n + 1) / 2
// * thus, the missing number is the difference of the exexpted sum and the actual sum
pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let n: u64 = input.parse();
    let numbers: Vec<u64> = input.parse_vec();

    out.println(do_solve(n, &numbers));
}

fn sum_u64(n: u64) -> u64 {
    n * (n + 1) / 2
}

fn do_solve(n: u64, numbers: &[u64]) -> u64 {
    let actual = numbers.iter().sum::<u64>();
    let expected = sum_u64(n);
    expected - actual
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(4, do_solve(5, &[2, 3, 1, 5]))
    }
}

