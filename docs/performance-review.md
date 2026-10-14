# Performance review guide

Tiger Style treats performance as design work first. Use this checklist to make workload, resource, and predictability assumptions reviewable before code turns into benchmark cargo cult. Static lints can catch local smells and benchmarks can validate a chosen workload, but neither replaces workload reasoning: reviewers still need explicit numbers, bounds, and hot-path hypotheses in plain language.

## Workload and hot path

Start by naming the exact entry point, steady-state load, peak load, and latency or throughput target. The biggest wins usually come from choosing a better data path, storage shape, or batching strategy before the profiler says anything.

- Reviewer prompt: What exact request path, loop, or batch is expected to dominate runtime or cost?
- Reviewer prompt: Which target matters here first: tail latency, throughput, memory footprint, or bounded queue depth?
- Checklist: Write down one steady-state workload and one peak case before discussing micro-optimizations.

## Napkin math

Napkin math matters because it turns vague intuition into a rough bound early. Use rounded numbers, stay honest about units, and aim to get within one order of magnitude, not to predict the fourth decimal place.

- Reviewer prompt: What are the three or four input numbers that drive this path: requests per second, bytes per item, retention period, batch size, or round trips?
- Reviewer prompt: If you multiply those numbers now, what daily volume, queue growth, or end-to-end latency do you get?
- Checklist: Do at least one throughput or storage calculation and one latency budget split before implementation.
- Example: `1 KB/event * 50,000 events/s ~= 50 MB/s ~= 4.3 TB/day` before replication or retention multipliers.

## Resource sketch: network, disk, memory, CPU

Performance review stalls when everyone says "fast" without naming the resource. Sketch each resource separately. Network and disk often dominate end-to-end latency; memory limits batching and cache fit; CPU limits hot inner-loop throughput. Each resource needs both bandwidth and latency reasoning, because high throughput with bad tail latency is still failure.

- Reviewer prompt: Network: bytes per request or batch, round trips, and acceptable p95 or p99 latency?
- Reviewer prompt: Disk: sync frequency, bytes written or read, and latency sensitivity if storage stalls?
- Reviewer prompt: Memory: working-set size, queue residency, cache fit, and maximum allowed resident bytes?
- Reviewer prompt: CPU: operations or scans per item, branch-heavy paths, and expected work per batch?
- Checklist: Check resources in Tiger Style priority order - network, disk, memory, CPU - but keep the actual bottleneck explicit.

## Cost × frequency ranking

A rare expensive control task might matter less than a cheap copy inside a million-iteration loop. Ranking by cost × frequency keeps review focused on the path that burns the most total time or bytes, not the path that merely sounds scary.

- Reviewer prompt: Which operation is hottest once you multiply local cost by call frequency?
- Reviewer prompt: Which expensive step is off the hot path and therefore lower priority right now?
- Checklist: List the top three cost × frequency items and say why lower-ranked items can wait.

## Control plane vs data plane

Control-plane work can afford richer coordination and lower frequency. Data-plane work usually needs fewer branches, fewer allocations, lower per-item overhead, and tighter latency bounds. Reviews go sideways when both planes share the same abstractions and pay the same cost.

- Reviewer prompt: Which operations are rare coordination or reconfiguration, and which ones run on every request or message?
- Reviewer prompt: Are control-plane checks, logging, or metadata lookups leaking into the data-plane hot path?
- Checklist: Keep control-plane flexibility out of the data-plane inner loop unless the measured workload truly needs it there.

## Batching, limits, queues, and backpressure

Queues and buffers look harmless until they grow without bound or serialize work into tiny expensive units. This section matters because batch size, queue limit, and overflow behavior often decide whether the system stays smooth under burst load or falls apart.

- Reviewer prompt: What is the explicit max item count or byte bound for each queue, buffer, channel, or collector on this path?
- Reviewer prompt: What batch size or flush trigger amortizes work without breaking latency goals?
- Reviewer prompt: If producers outrun consumers, what is the cost × frequency of backlog growth in bytes, CPU work, or added latency?
- Reviewer prompt: If producers outrun consumers, what backpressure, shedding, or failure mode keeps memory and latency bounded?
- Checklist: Every queue gets a bound, every batch gets a trigger, and every overload path says what happens next.

## CPU predictability and compiler luck

CPU work is cheapest when it runs in large predictable chunks. Tight loops with primitive arguments, clear memory access patterns, and simple branches give the compiler and hardware an easy job. If a hot path only performs well when the optimizer guesses right, the design is fragile.

- Reviewer prompt: Can the hot inner loop consume a batch in one pass instead of bouncing through per-item calls?
- Reviewer prompt: Does the inner loop take primitive arguments or slices, or is it dragging `&self`, trait objects, or large structs through each iteration?
- Reviewer prompt: Are branch patterns, cache access, and allocation behavior predictable enough to stay stable under real load?
- Checklist: Prefer obvious tight loops and large work chunks over clever code that needs compiler luck to become fast.

## Measurement and validation

Sketches are for choosing direction. Measurement is for checking whether the real system matches the sketch. Once the design is clear, validate the dominant hypothesis with benchmarks, production-like load, profiling, or counters that match the workload you claimed.

- Reviewer prompt: Which metric or counter will tell you whether the sketch was right: throughput, tail latency, queue depth, memory peak, or CPU time?
- Reviewer prompt: What workload will you replay or simulate, and how close is it to the expected real traffic?
- Checklist: Compare measurements back to the napkin sketch and rewrite the design if the real bottleneck lands elsewhere.
