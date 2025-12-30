// https://cses.fi/problemset/task/1158
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    let fl: Vec<usize> = input.parse_vec();
    if fl.len() != 2 {
        panic!("expected two initial inputs: n and x");
    }
    let prices: Vec<usize> = input.parse_vec();
    let pages: Vec<usize> = input.parse_vec();

    let result = max_pages(fl[1], &prices, &pages);

    out.println(result);
}

fn max_pages(x: usize, b_prices: &[usize], b_pages: &[usize]) -> usize {
    assert_eq!(b_prices.len(), b_pages.len());

    // a tally line of maximum pages at each price point.
    let mut tally = vec![0u32; x + 1];

    // visit each book and merge it into the tally line back to front.
    for (&price, &pages) in b_prices.iter().zip(b_pages.iter()) {
        // for a price point that is unaffordable, the range oeprator would produce an empty range.
        for i in (price..=x).rev() {
            // the recurrance relation builds on the maximum pages currently known for the remaining
            // budget: i-price. Going back to front ensures that each book is only counted once.
            let remaining_budget = i - price;
            tally[i] = tally[i].max(tally[remaining_budget] + pages as u32);
        }
    }

    tally[x] as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        // Example: n=4, x=10, prices=[4,8,5,3], pages=[5,12,8,1]
        // Optimal: buy books 1 and 3 for 4+5=9 price and 5+8=13 pages
        assert_eq!(max_pages(10, &[4, 8, 5, 3], &[5, 12, 8, 1]), 13);
    }
}
