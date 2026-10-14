## Context

A function can look pure while mutating through `&self` when its data type owns cells, locks, atomics, or `UnsafeCell`. Octet should surface those shapes in declared core scopes without banning synchronization at system boundaries.

## Decisions

### 1. Check core-scope declarations and construction

**Choice:** Emit for interior-mutability types used as fields or directly constructed inside `core_source_scopes`.

**Rationale:** These shapes are visible, high signal, and close to the design decision.

### 2. Leave shell synchronization alone

**Choice:** Do not emit outside configured core scopes unless a later pure-marker contract requests stricter behavior.

**Rationale:** Mutexes and atomics are valid shell/concurrency tools.

### 3. Use type-path classification first

**Choice:** Start with path/type classification for `Cell`, `RefCell`, `UnsafeCell`, `Mutex`, `RwLock`, and `Atomic*` types; add resolved-path improvements behind fixtures.

**Rationale:** This keeps implementation small and reviewable.

## Risks / Trade-offs

- Some core algorithms use atomics for lock-free data structures. They should either live outside pure scopes or carry justified suppressions.
- Type aliases can hide interior mutability. Initial direct-type coverage is acceptable if documented.

## Validation

Use pure classifier unit tests and UI fixtures for both emitting and clean source shapes.
