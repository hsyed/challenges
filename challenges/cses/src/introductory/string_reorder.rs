// https://cses.fi/problemset/task/1743
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let s = input.next_line();

    // My initial thought was to simply sort the string and
    // then making shifts till there are no gaps.
    //
    // The correct solution is to count the characters and
    // then build up the result incrementally.
    //
    // The characters are counted up front as this:
    // 1. determines we can produce a valid string.
    // 2. use the character counts to guide the building process.
    let result = if let Some(res) = do_solve(&s) {
        res
    } else {
        "-1".to_string()
    };
    out.println(result);
}

fn do_solve(input: &str) -> Option<String> {
    let n = input.chars().count() as u32;
    let mut tally = [0_u32; 26];
    let mut last_char = None;
    let mut result = String::with_capacity(n as usize); //

    // build tally and validate thje shape
    {
        input.chars().for_each(|c| {
            assert!(c.is_ascii_uppercase());
            let index = (c as u8 - b'A') as usize;
            tally[index] += 1;
        });

        let threshold = n.div_ceil(2);
        if tally.iter().any(|&c| c > threshold) {
            return None;
        }
    }

    for remaining in (1..=n).rev() {
        // Find the character to use:
        // 1. If a character has count > remaining/2, we MUST use it
        // 2. Otherwise, use the smallest character != last_char
        let threshold = remaining / 2;
        let forced_idx = tally.iter().position(|&count| count > threshold);

        let chosen_idx = if let Some(idx) = forced_idx {
            // Must use this high-frequency character
            idx
        } else {
            // Pick smallest character != last_char
            let mut best = None;
            for (i, &count) in tally.iter().enumerate() {
                if count > 0 && Some(i as u8) != last_char.map(|c| c - b'A') {
                    best = Some(i);
                    break;
                }
            }
            best?
        };

        let ch = b'A' + chosen_idx as u8;
        result.push(ch as char);
        tally[chosen_idx] -= 1;
        last_char = Some(ch);
    }
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(do_solve("HATTIVATTI"), Some("AHATITITVT".to_string()));
    }

    #[test]
    fn test_impossible() {
        assert_eq!(do_solve("AAA"), None);
    }
}
