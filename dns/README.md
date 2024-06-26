# DNS forwarder challenge

Solution for the [DNS forwarder challenge](https://codingchallenges.fyi/challenges/challenge-dns-forwarder).

## Constraints

* Only support clients advertising EDNS0 support and a UDP payload size of 4096 bytes.
* Only single question queries are cached / enforce only single question queries.

## Notes:

* Wireshark is the best resource for DNS packet structure. Alternative [this](https://www.catchpoint.com/blog/how-dns-works) is also good guide.
* Dig (on OS X) reports the recipient UDP size as 512 but sends 4096.

## Status

Learned a lot about DNS and Tokio. I'd consider the challenge complete in the current state. A comprehensive DNS library 
already [exists](https://github.com/hickory-dns/hickory-dns) so its best to stop here.

For more learning, to continue from here the first thing I would do is to port the guts of the IO handling to 
[Tokio tower services](https://docs.rs/tower/latest/tower/trait.Service.html). I've been implementing the necessary 
patterns directly. Other things I'd work on: 

* Implementing a cache eviction policy. Currently, the cache is unbounded on size.
* Return error packets to the client for IO errors in all cases.
* Improving error handling in the protocol module. I don't think relevant context is available when packets are malformed.
* Consider pooling the `MessageWriter`. It allocates a HashMap tally and is also used with a vector.
* Logging.
* Metrics.
* Support >1 packet dispatch tasks / investigate thread-per-core for this?.







