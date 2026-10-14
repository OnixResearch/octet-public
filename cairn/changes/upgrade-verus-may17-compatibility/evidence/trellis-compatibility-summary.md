# Trellis compatibility observation

A local closed-portfolio run completed with the May 17 Octet package before the Octet revision was committed:

- reviewed Trellis baseline: `10712 verified, 0 errors`, exit `0`;
- May 17 complete Trellis: `10712 verified, 0 errors`, exit `0`;
- adapted IronKV: `312 verified, 0 errors`, linked exit `0`;
- adapted verified-memory-allocator: `732 verified, 0 errors`, linked exit `0` against candidate-Rust libc 0.2.147;
- unmodified verified-node-replication: `254 verified, 0 errors`, exit `0`;
- timeouts: none;
- added assumptions, external bodies, trusted markers, or triggers: none.

The provisional receipt has BLAKE3 `b00ea7cadf9514e4bc4dfa6a419ee9b7a91be31de706da33bc2c4b397ce4a0ad`. It correctly recommends the Trellis compatibility transition from `local-pin` to `native`.

This receipt is not the final cross-repository artifact because its matrix still names the previously published Octet provider revision. Trellis must rerun after this Octet change is committed, bind the resulting exact Octet revision and native profile row, and preserve its own receipt authority. Adapted-source passes do not certify the unmodified upstream revisions or authorize runtime adoption.
