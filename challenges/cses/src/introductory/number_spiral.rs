//https://cses.fi/problemset/task/1071
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let t: usize = input.parse();

    for _ in 0..t {
        let parts: Vec<u64> = input.parse_vec();
        assert_eq!(parts.len(), 2);
        let result = do_solve(parts[0], parts[1]);
        out.println(result);
    }
}

fn do_solve(y: u64, x: u64) -> u64 {
    // 1  2  9  10 25
    // 4  3  8  11 24
    // 5  6  7  12 23
    // 16 15 14 13 22
    // 17 18 19 20 21

    // 1. y an x lie on the boundary of some square of order n.
    // 2. Even order squares are filled clockwise, odd order squares counter-clockwise.
    // 3. order n always begins after (n-1)^2 and ends at n^2.
    let n = y.max(x);
    let clockwise = n.is_multiple_of(2);
    // we can flip the polarity of x and y which would use 2 cases instead of the 4 below.
    if clockwise {
        if y == n {
            // on the bottom row
            n * n - (x - 1)
        } else {
            // on the right column
            (n - 1) * (n - 1) + y
        }
    } else if x == n {
        // on the right column
        n * n - (y - 1)
    } else {
        // on the bottom row
        (n - 1) * (n - 1) + x
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_examples() {
        // From problem statement
        assert_eq!(do_solve(2, 3), 8);
        assert_eq!(do_solve(1, 1), 1);
        assert_eq!(do_solve(4, 2), 15);
    }
}
