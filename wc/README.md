# Word count tool

Solution to [the wc tool challenge](https://codingchallenges.fyi/challenges/challenge-wc).

Main thing I learned during this exercise was how to use the [Clap CLI parser](https://github.com/clap-rs/clap) and how 
to organise the visitor pattern in Rust. For the sake of efficiency, a proper implementation will be flat and def not 
use dynamic dispatch. 