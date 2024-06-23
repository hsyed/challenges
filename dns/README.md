# DNS forwarder challenge

Solution for the [DNS forwarder challenge](https://codingchallenges.fyi/challenges/challenge-dns-forwarder).

## Notes:

* Wireshark is the best resource for DNS packet structure. Alternative [this](https://www.catchpoint.com/blog/how-dns-works) is also good guide.
* Dig (on OS X) reports the recipient UDP size as 512 but sends 4096.

## Todo:

* Caching: supporting code is written, seems uninteresting for now.
* Return error packets to the client for IO errors, currently swallowed.
* Improving error handling in the protocol module. I don't think relevant context is available when packets are malformed.
* Harden the service against panics on socket breaks -- re-establish connectivity. 
