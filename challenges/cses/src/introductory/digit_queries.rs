//https://cses.fi/problemset/task/2431
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let q: usize = input.parse();

    for _ in 0..q {
        let k: u64 = input.parse();
        let result = do_solve(k);
        out.println(result);
    }
}

fn do_solve(mut k: u64) -> u64 {
    // digits = number of digits in current group (1,  2,   3, ...)
    // count = how many numbers in this group     (9, 90, 900, ...)
    // start = first number in this group         (1, 10, 100, ...)
    let mut digits: u64 = 1;
    let mut count: u64 = 9;
    let mut start: u64 = 1;

    // Find which group of d-digit numbers contains position k
    while k > digits * count {
        k -= digits * count;
        digits += 1;
        start *= 10;
        count *= 10;
    }

    // Now k is the position within the group of `digits`-digit numbers
    // Find which specific number: start + how many complete numbers we skip
    let number = start + (k - 1) / digits;

    // Find which digit within that number (0-indexed from left)
    let digit_index = (k - 1) % digits;

    // Extract the digit at that position
    extract_digit(number, digits, digit_index)
}

fn extract_digit(number: u64, total_digits: u64, digit_index: u64) -> u64 {
    // digit_index is 0-indexed from the left
    // We need to get the digit at position digit_index
    let divisor = 10u64.pow((total_digits - digit_index - 1) as u32);
    (number / divisor) % 10
}

#[allow(dead_code)]
// first approach: itterate over n at a time with no optimizations
fn do_solve_brute(n: &usize) -> u32 {
    let mut cur: usize = 1;
    let mut rem = *n;
    let mut cur_size = 1;

    loop {
        if rem > cur_size {
            // the current number can be iterated over
            cur += 1;
            rem -= cur_size;
            cur_size = count_digits(&cur);
        } else {
            break;
        }
    }

    digit_at_pos(&cur, &cur_size, &rem)
}

#[inline(always)]
fn count_digits(i: &usize) -> usize {
    if *i == 0 {
        1
    } else {
        (*i as f32).log10().floor() as usize + 1
    }
}

#[inline(always)]
fn digit_at_pos(n: &usize, num_digits: &usize, pos: &usize) -> u32 {
    let divisor = 10usize.pow((*num_digits - *pos) as u32);
    ((*n / divisor) % 10) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brute_simple() {
        let test_cases = vec![
            (1, 1),
            (9, 9),
            (10, 1), // 10
            (11, 0), // 10
            (12, 1), // 11
            (13, 1), // 11
            (14, 1), // 12
            (15, 2), // 12
        ];

        for (input, expected) in test_cases {
            let result = do_solve_brute(&input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_optimal_simple() {
        let test_cases = vec![
            (1, 1),
            (9, 9),
            (10, 1), // 10
            (11, 0), // 10
            (12, 1), // 11
            (13, 1), // 11
            (14, 1), // 12
            (15, 2), // 12
        ];

        for (input, expected) in test_cases {
            let result = do_solve(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    #[test]
    fn test_optimal_with_large_values() {
        let test_cases = vec![
            (189, 9),  // Last digit of 99
            (190, 1),  // First digit of 100
            (191, 0),  // Second digit of 100
            (192, 0),  // Third digit of 100
            (1000, 3), // Somewhere in 3-digit range
        ];

        for (input, expected) in test_cases {
            let result = do_solve(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }
}
