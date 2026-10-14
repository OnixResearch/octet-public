# Remote effect contract

- capability: `<EndpointAuthority>`
- timeout: `<duration>`
- retry_policy: `<bounded attempts and backoff>`
- idempotency_key: `<nominal key type>`
- request_hash: `b3:<request-shape-hash>`
- receipt_identity: `<receipt id or response hash>`
- replay_fixture: `<path to deterministic handler fixture>`
- acceptance: retries are bounded and safe because the idempotency key is stable for the requested effect.
