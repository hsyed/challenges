//https://cses.fi/problemset/task/1746
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let parts: Vec<usize> = input.parse_vec();
    if parts.len() != 2 {
        panic!("expected two initial inputs: n and x");
    }
    let n: usize = parts[0];
    let m: usize = parts[1];
    let arr: Vec<u8> = input.parse_vec();

    let result = do_solve(m, &arr);

    out.println(result);
}

fn do_solve(m: usize, desc: &[u8]) -> u32 {
    let n = desc.len();
    let mut tally = vec![vec![0u32; m + 1]; n];

    // init first column
    if desc[0] == 0 {
        // element 0 is always 0
        (1..=m).for_each(|i| {
            tally[0][i] = 1;
        });
    } else {
        tally[0][desc[0] as usize] = 1;
    }

    (1..n).for_each(|i| {
        if desc[i] == 0 {
            (1..=m).for_each(|cur| {
                step(m, &mut tally, i, cur);
            });
        } else {
            let cur = desc[i] as usize;
            step(m, &mut tally, i, cur);
        }
    });

    // last column either contains all possible ends, or a fixed end. Summing them
    // all up works for both cases.
    (1..=m).fold(0u32, |acc, v| {
        (acc + tally.last().unwrap()[v]) % 1_000_000_007
    })
}

// make one step in the DP table tally[i][cur]
#[inline(always)]
fn step(m: usize, tally: &mut [Vec<u32>], i: usize, cur: usize) {
    tally[i][cur] = (tally[i][cur] + tally[i - 1][cur]) % 1_000_000_007;
    if cur > 1 {
        tally[i][cur] = (tally[i][cur] + tally[i - 1][cur - 1]) % 1_000_000_007;
    }
    if cur < m {
        tally[i][cur] = (tally[i][cur] + tally[i - 1][cur + 1]) % 1_000_000_007;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // TODO: Add test cases
        assert_eq!(3, do_solve(5, &[2, 0, 2]));
    }
}
