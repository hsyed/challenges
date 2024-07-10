# Memcached challenge

This is a solution for the [memcached challenge](https://codingchallenges.fyi/challenges/challenge-memcached).

The main goal for this challenge is to practice how an idiomatic async rust TCP service fits together. The Tokio project 
has a [solution](https://github.com/tokio-rs/mini-redis) to the [Redis challenge](https://codingchallenges.fyi/challenges/challenge-redis/)
here. I'm going to use this as a reference.

## Things learned from this challenge:

* in a function signature like so: `run(tcp_listener: TcpListener, shutdown: impl Future)`, the `impl` is syntactic 
sugar for generics. The compiler will generate a unique function for each distinct call site. I guess this is acceptable when
there is a reasonable bound on the number of distinct call sites? The alternative is using `dyn`.
* A single source tree can be organized to contain multiple binaries. This is done by adding a `[[bin]]` section to 
`Cargo.toml`.
* Tokio contains all the machinery for wiring up the shutdown sequence. A shutdown signal is available as a future which 
then has to be wired up to a broadcast channel to gracefully shutdown background activities -- e.g., client connection 
handlers.
* `mpsc` channels can be used to co-ordinate shutdown. The `Sender` is cloned and passed to all tasks. It is safe to 
shut down when all `Senders` are dropped, as the `Receiver` will "close" signalled by the return of `None`.

### Libs

* [Tokio tracing](https://github.com/tokio-rs/tracing): Tokio tracing support. Includes a backend for the rust log 
package.
* [Moka](https://github.com/moka-rs/moka): a concurrent caching library. This does not support synchronous visits so we 
take a write lock when performing storage operations.     

### TODO

* wire up prometheus metrics.
* wire up otel tracing.
* Make the cache max weighted capacity configurable.