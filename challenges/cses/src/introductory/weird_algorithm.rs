//https://cses.fi/problemset/task/1068
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let n: u64 = input.parse();
    let result = do_solve(&n);
    out.println(result);
}

fn do_solve(n: &u64) -> String {
    let mut n: u64 = *n;
    let mut result = String::new();

    loop {
        result.push_str(&n.to_string());
        if n == 1 {
            break;
        }
        result.push(' ');
        n = if n.is_multiple_of(2) {
            n / 2
        } else {
            n * 3 + 1
        };
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(do_solve(&7), "7 22 11 34 17 52 26 13 40 20 10 5 16 8 4 2 1");
    }
}

