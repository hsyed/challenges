//https://cses.fi/problemset/task/1634
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let parts: Vec<usize> = input.parse_vec();

    let [_, x] = parts[..] else {
        panic!("Expected 2 values")
    };

    let coins: Vec<usize> = input.parse_vec();

    let result = min_coins(x, &coins);

    out.println(result);
}

// Sentinel value representing the impossible state.
const IMPOSSIBLE: usize = usize::MAX;

// Dynamic programming solution: builds up solutions for sums 1..=target incrementally.
//
// For each sum i, try each coin c as the "last coin" in the solution:
//   - If we can make (i - c), then we can make i using that solution + coin c
//   - tally[i] = min(tally[i], tally[i - c] + 1) for all valid coins c
//
// The itterative approach 1 element at a time ensures that every i - current is a valid index.
fn min_coins(target: usize, coins: &[usize]) -> i32 {
    // All sums start as IMPOSSIBLE until we find a way to construct them
    let mut tally = vec![IMPOSSIBLE; target + 1];

    // Base case: making sum 0 requires 0 coins (the empty set of coins).
    // This is the foundation - when we use a coin equal to the target sum,
    // tally[target - coin] will be tally[0] = 0, giving us a valid solution of 1 coin.
    tally[0] = 0;

    for current in 1..=target {
        for coin in coins {
            if *coin <= current {
                // Look up how many coins needed to make the remainder after using this coin.
                // Note: tally_remainder can be 0 when coin == current (using exact coin match),
                // which is valid - it means we can make 'current' with just this one coin.
                let tally_remainder = tally[current - coin];
                if tally_remainder != IMPOSSIBLE {
                    let candidate = tally_remainder + 1; // +1 to include this coin!
                    if candidate < tally[current] {
                        tally[current] = candidate;
                    }
                }
            }
        }
    }
    match tally[target] {
        n if n != IMPOSSIBLE => n as i32,
        _ => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Example: coins = [1, 5, 7], target = 11
        // Optimal: 5 + 5 + 1 = 11 (3 coins)
        assert_eq!(min_coins(11, &[1, 5, 7]), 3);
    }
}
