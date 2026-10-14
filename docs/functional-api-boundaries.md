# Functional API boundary guidance

Octet's useful functional lesson is type-level boundary design, not point-free cleverness. Rust generics, associated types, sealed traits, and marker states can move protocol and state errors from runtime checks into compile errors.

## Use type-level separation when state is known

Prefer a type-level boundary when:

- a protocol-specific method is valid for one protocol but invalid for another;
- a state transition exposes capabilities only after acquisition, validation, or authentication;
- a format, adapter, or plugin API has separate producer and consumer roles;
- callers currently receive `Option` or enum variants even when the code path knows the state statically.

Good shapes include:

- `Request<Nfs>` and `Request<Bootp>` instead of one request with optional protocol fields;
- sealed traits for internal protocol families so downstream users cannot invent invalid states;
- associated types for protocol-specific auth, handles, errors, or capabilities;
- marker-state APIs such as `Session<Unauthenticated>` -> `Session<Authenticated>`.

## Visitor, prism, and optics-style seams

Optics-style designs are advanced boundary tools. Use them when two independent axes need to compose without every side knowing the other's concrete type, such as serialization formats versus domain types, traversal visitors versus data structures, or plugin adapters versus typed core models.

Review requirements for these seams:

- name both axes being decoupled;
- keep generated or macro-heavy bridge code behind a small API;
- state the error and recovery path for failed conversion;
- provide fixtures for at least two types and two formats/adapters before claiming the abstraction works;
- do not expose lifetime-erased internals as a normal API.

## Declarative code limits

Iterator chains, `fold`, and combinators are acceptable when they are short, bounded, and easier to audit than a loop. Prefer an explicit loop when:

- the path is hot and branch/cache behavior matters;
- bounds, early exits, or backpressure must be visible;
- assertion density would disappear inside closures;
- error handling needs step-specific context;
- the chain requires readers to simulate types or side effects mentally.

Functional style should make invalid states unrepresentable. It should not make valid code unreadable.
