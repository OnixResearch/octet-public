# Proof-to-implementation gap

A verified model is not a verified implementation.

Octet records one named failure class at the proof-to-implementation boundary:
a **specification assumption** that the verified model treats as true and that
the running system does not guarantee. A defect in this class exists when the
implementation violates the assumption. Every model-level proof can still
pass, because the proof reasons about the model and not about the deployed
process, its scheduler, its storage, or its extensions.

The class gives reviewers a shared vocabulary for that situation. It is not a
verdict about any particular proof, and it is not an Octet check.

## The four specification assumptions

1. Synchronous process assumption
2. Request-to-response correlation assumption
3. Atomic persistence of related protocol state
4. Formal coverage of the entire protocol, including extensions

Each assumption is a statement about the world that the model relies on. A
formal proof that carries the assumption is correct about the model. When the
assumption is false in the running system, the proof's conclusion stops
describing the implementation.

## The three layers

The class lives in three layers. Each layer has its own defect shape and its
own closing evidence.

| Layer | Assumption at risk | What the model cannot see | What closes the layer |
| --- | --- | --- | --- |
| interleaving | Synchronous process assumption, Request-to-response correlation assumption | that the implementation runs several activities at once, and that a reply can be observed by the wrong caller | system-level interleaving testing |
| fault handling | Atomic persistence of related protocol state | that the process can stop between two writes, and that storage can be read back half-updated | system-level fault injection testing |
| unverified extensions | Formal coverage of the entire protocol, including extensions | that the deployed protocol set is larger than the formalized protocol set | extension conformance and differential testing |

### Interleaving

The Synchronous process assumption models the implementation as one process that
does one thing at a time. The Request-to-response correlation assumption models
each reply as belonging to the request that caused it.

A real implementation breaks both assumptions at once. Concurrent requests,
background work, retries, and connection reuse make the observable order of
events differ from the modeled order, and a reply produced for one request can
be delivered, cancelled, or attributed to another. A model-level proof cannot
see this: it reasons about the modeled order and never observes the scheduler.

What closes this layer is system-level interleaving testing of the running
implementation: schedule-controlled concurrent execution of the real system,
with the recorded history checked for the correlation and ordering properties
the model assumed. The evidence is a replayed execution history, not a model
checking run.

### Fault handling

The Atomic persistence of related protocol state assumption models related
protocol state as updated together or not at all. The running system can stop
between two writes, lose a write, or expose a partially updated state to a
reader.

A model-level proof cannot see this: it assumes an atomic step and never
observes a process that is not running. What closes this layer is system-level
fault injection testing: terminate, restart, and truncate the real system at
its persistence and I/O boundaries, then check that no reader ever observes a
half-updated related state and that recovery converges to a state the protocol
accepts. The evidence is a fault-injection run against the implementation.

### Unverified extensions

The Formal coverage of the entire protocol, including extensions assumption
models the formalized protocol as the whole protocol. The deployed protocol set
is usually larger: optional extensions, negotiated features, and versioned
variants are implemented without a corresponding proof.

A model-level proof cannot see this: formal coverage is a property of the
model, and the extensions are outside the model by construction. What closes
this layer is extension conformance and differential testing: run every
extension against the same base protocol contract and compare its observable
behavior with the base, so that the uncovered part of the protocol is covered by
executable evidence instead of by an assumption.

## What closes the layers, and what does not

- Property-based and law tests over a pure core ([law-tested deterministic
  cores](law-tested-cores.md), [Hegel property
  evidence](hegel-property-evidence.md)) prove pure-core invariants. They do
  not close these layers: the layers are properties of a running system's
  scheduling, failure, and extension behavior, not of a pure function.
- Static checks, lints, and policy checks report source shape and declared
  policy. They do not observe runtime interleaving, runtime failure, or
  extension behavior.
- Scoped formal proof receipts (`cargo octet prove`) support their stated
  property under the recorded model, verifier, and assumptions. They do not
  extend past the model, and they do not describe the deployed system.
- Only system-level fault testing of the running implementation closes the
  interleaving, fault handling, and unverified extension layers.

## Reporting the class

When an implementer or a reviewer finds a defect in this class, record:

- the assumption that was violated, named exactly as listed above;
- the layer: interleaving, fault handling, or unverified extensions;
- the observed implementation behavior and the modeled behavior that differs;
- the model-level property that still holds, so the reader can see why the
  proof did not fail;
- the system-level test that reproduces the defect, or the open obligation when
  no such test exists yet.

A report that names the assumption and the layer is a claim about the
implementation boundary. It is not a claim that the proof is wrong.

## Claim boundary

- Documenting this class does not claim that any existing proof, in this
  repository or in a consumer repository, is unsound. A proof that states an
  assumption is correct about the model that states it.
- Documenting this class does not claim that system-level interleaving,
  fault-injection, or extension-conformance testing exists for this repository
  today. Octet's checked path runs source-shape, policy, fixture, transcript,
  law, and optional verifier-backed rails. Those rails do not close these three
  layers, and this repository does not currently run a system-level fault
  campaign for them.
- The class records where a proof's reach ends. It does not extend a proof to
  the implementation, and it does not replace the review that a proof receipt
  already requires.
- The class is not an Antithesis capability claim, not a coverage claim, and
  not a certification. It is the vocabulary and the boundary for one named
  defect shape.

Read the [trust-boundary and evidence path](trust-boundary-evidence.md) for the
evidence tiers and non-claims that surround this class.
