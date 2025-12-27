// https://cses.fi/problemset/task/1635
//   coins = {2, 3, 5}, target = 9:
//
//   dp[0]                                 = 1
//   dp[1]                                 = 0
//   dp[2]                                 = 1  → {2}
//   dp[3] = dp[1] + dp[0]         = 0 + 1 = 1  → {3}
//   dp[4] = dp[2] + dp[1]         = 1 + 0 = 1  → {2,2}
//   dp[5] = dp[3] + dp[2] + dp[0] = 1+1+1 = 3  → {2,3},     {3,2},   {5}
//   dp[6] = dp[4] + dp[3] + dp[1] = 1+1+0 = 2  → {2,2,2},   {3,3}
//   dp[7] = dp[5] + dp[4] + dp[2] = 3+1+1 = 5  → {2,2,3},   {2,3,2}, {3,2,2}, {2,5},   {5,2}
//   dp[8] = dp[6] + dp[5] + dp[3] = 2+3+1 = 6  → {2,2,2,2}, {3,3,2}, {2,3,3}, {3,2,3}, {5,3}, {3,5}
//   dp[9] = dp[7] + dp[6] + dp[4] = 5+2+1 = 8  ✓ (shown below)
//
//   The 8 ways to make 9:
//   - {2,2,2,3}: 4 permutations →  {2,2,2,3}, {2,2,3,2}, {2,3,2,2}, {3,2,2,2}
//   - {2,2,5}:   3 permutations →  {2,2,5},   {2,5,2},   {5,2,2}
//   - {3,3,3}:   1 permutation →   {3,3,3}
//
//   Total: 4 + 3 + 1 = 8 ways ✓

use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let line: Vec<usize> = input.parse_vec();
    let _n = line[0];
    let x = line[1];
    let coins: Vec<usize> = input.parse_vec();

    let result = do_solve(x, &coins);
    out.println(result);
}

fn do_solve(x: usize, coins: &[usize]) -> usize {
    let mut combinations = vec![0; x + 1];
    combinations[0] = 1; // there is one way to select n when the coin is the same as the target.

    for i in 1..=x {
        for coin in coins {
            if i >= *coin {
                combinations[i] += combinations[i - coin];
                combinations[i] %= 1_000_000_007;
            }
        }
    }
    combinations[x] % 1_000_000_007
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_stub() {
        assert_eq!(8, super::do_solve(9, &[2, 3, 5]));
    }
}
