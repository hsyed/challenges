# DNS forwarder challenge

Solution for the [DNS forwarder challenge](https://codingchallenges.fyi/challenges/challenge-dns-forwarder).

## Constraints

* Only support clients advertising EDNS0 support and a UDP payload size of 4096 bytes.
* Only single question queries are cached / enforce only single question queries.

## Notes:

* Wireshark is the best resource for DNS packet structure. Alternative [this](https://www.catchpoint.com/blog/how-dns-works) is also good guide.
* Dig (on OS X) reports the recipient UDP size as 512 but sends 4096.

## Todo:

* Return error packets to the client for IO errors, currently swallowed.
* Improving error handling in the protocol module. I don't think relevant context is available when packets are malformed.
* Harden the service against panics on socket breaks -- re-establish connectivity.
* Consider pooling the `MessageWriter`. It allocates a HashMap tally and is also used with a vector.
* Support >1 packet dispatch tasks / investigate thread-per-core for this?
