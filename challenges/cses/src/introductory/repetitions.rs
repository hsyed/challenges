//https://cses.fi/problemset/task/1069
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let s: String = input.parse();
    out.println(do_solve(&s));
}

fn do_solve(s: &str) -> usize {
    // start with any valid char .. if it lands on the right char it
    // will +1 , if not it will switch to the correct char and set to 1;
    let mut largest_run = 0;
    let mut current_run: usize = 0;
    let mut current_char = 'A';

    for c in s.chars() {
        if c == current_char {
            current_run += 1;
        } else {
            current_char = c;
            current_run = 1;
        }

        if current_run > largest_run {
            largest_run = current_run;
        }
    }
    largest_run
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(3, do_solve("ATTCGGGA"));
    }
}
