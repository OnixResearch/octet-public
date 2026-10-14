# Performance sketch template

## Workload or entry point

- Artifact or path under review:
- Owner / reviewer:
- Steady-state workload:
- Peak workload or burst case:
- Tail-latency or throughput target:
- Hot path summary:

## Assumptions and units

- Time unit used:
- Byte or item unit used:
- Replication, durability, or redundancy assumptions:
- Concurrency assumptions:
- Simplifying assumptions to revisit:
- Unknown input still blocking estimate:

## Bandwidth and latency sketch

- Operation or batch being estimated:
- Requests or items per second:
- Bytes per item or batch:
- Round trips or sync points:
- Estimated bandwidth needed:
- Estimated end-to-end latency budget:
- Largest term in that latency budget:

## Resource budgets or limits

- Network budget or limit:
- Disk budget or limit:
- Memory budget or limit:
- CPU budget or limit:
- Queue or buffer byte limit:
- Other resource budget or limit:

## Cost × frequency ranking

- Highest total cost item (cost × frequency):
- Second-ranked item:
- Third-ranked item:
- Rare but expensive work to defer:
- Why lower-ranked items can wait:

## Control-plane vs data-plane split

- Control-plane operations here:
- Data-plane operations here:
- Shared code or state crossing both planes:
- Cost that must move out of data plane:
- Reason current split is acceptable or next change needed:

## Batching, limits, queues, and backpressure

- Batch unit and size:
- Flush trigger or release condition:
- Queue, channel, or buffer names:
- Max items or bytes per queue:
- Producer rate vs consumer rate estimate:
- Cost × frequency risk if backlog grows:
- Backpressure or overload behavior:
- Failure mode if the bound is hit:

## Measurement or validation plan

- First metric to measure:
- Workload generator or trace:
- Benchmark, profile, or counter plan:
- Expected result from sketch:
- Result that would prove the sketch wrong:
- Next measurement after first pass:

## Open risks and non-goals

- Biggest assumption likely to be wrong:
- Known workload not covered yet:
- Deliberate non-goal:
- Follow-up experiment or design change:
- Review verdict or next action:
