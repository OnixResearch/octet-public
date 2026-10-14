# Cairn COB collaboration

Octet can place one native Cairn change in Cairn's local signed Git COB format without joining a Radicle network. This is an operator-selected collaboration backend, not an Octet runtime dependency.

Use the sibling or packaged Cairn CLI that includes backend `radicle-cob.v1`. Every command must name the lifecycle root and Git COB repository explicitly. Mutation commands additionally require an explicit OpenSSH Ed25519 keystore directory containing `radicle` and `radicle.pub`.

## Boundaries

The provider:

- reads native change artifacts from `.cairn/changes/<change>/`;
- writes signed COB commits and namespaced refs to the selected Git repository;
- embeds the creator and optional delegate keys immutably at object creation;
- previews import, update, checkout, and resolution before mutation; and
- reports conflicts without choosing a winner.

The provider does **not** load Radicle profiles or identity documents, run a node, discover peers, seed, announce, connect, fetch, publish, synchronize, or prove convergence. Repository-local `user.name` and `user.email` supply Git display metadata only and grant no authority.

## Prepare explicit local state

Keep the signing key outside tracked source. The following example uses ignored `target/` paths and a separate bare store so Octet's primary `.git` refs remain unchanged:

```sh
mkdir -p target/cairn-cob/keys
chmod 700 target/cairn-cob/keys
ssh-keygen -q -t ed25519 -N '' -C octet-cob -f target/cairn-cob/keys/radicle

git clone --bare --no-local . target/cairn-cob/store.git
git --git-dir=target/cairn-cob/store.git remote remove origin
git --git-dir=target/cairn-cob/store.git config user.name 'Octet COB Operator'
git --git-dir=target/cairn-cob/store.git config user.email 'cob@octet.invalid'
```

An unencrypted key is suitable only for a disposable local pilot. Production operators must apply their own key custody and access policy. Neither Git display identity nor possession of unrelated host credentials grants COB authority.

## Import preview and execution

```sh
cairn cob import CHANGE \
  --root . \
  --git-repository target/cairn-cob/store.git \
  --keystore target/cairn-cob/keys \
  --workspace workspace:octet \
  --policy cairn-policy/generated/cairn-policy.json
```

Review `blocked`, `blockers`, `dry_run`, `mutated`, `actor`, `repository_head_oid`, `plan_hash`, `content_root`, artifact hashes, and non-claims. Execute the same bounded plan only after review:

```sh
cairn cob import CHANGE \
  --root . \
  --git-repository target/cairn-cob/store.git \
  --keystore target/cairn-cob/keys \
  --workspace workspace:octet \
  --policy cairn-policy/generated/cairn-policy.json \
  --execute
```

## Status and signed update

Inspect local divergence without mutation:

```sh
cairn cob status CHANGE \
  --root . \
  --git-repository target/cairn-cob/store.git \
  --policy cairn-policy/generated/cairn-policy.json
```

After changing active lifecycle artifacts, preview and review a signed update:

```sh
cairn cob update CHANGE \
  --root . \
  --git-repository target/cairn-cob/store.git \
  --keystore target/cairn-cob/keys \
  --policy cairn-policy/generated/cairn-policy.json
```

Add `--execute` only after the preview identifies the expected actor, basis, changed artifacts, content root, and non-claims. Ordinary update refuses conflicting heads; it never resolves them implicitly.

## Explicit Git transport

Cairn has no collaboration sync command. Move the base branch and COB refs with operator-owned Git transport. This local example deliberately configures no remote in the destination store:

```sh
git init --bare target/cairn-cob/second.git
git --git-dir=target/cairn-cob/second.git fetch --no-tags \
  target/cairn-cob/store.git \
  'refs/heads/main:refs/heads/main' \
  'refs/namespaces/*:refs/namespaces/*'
```

Compare exact targets before consuming the destination:

```sh
git --git-dir=target/cairn-cob/store.git for-each-ref \
  --format='%(refname) %(objectname)' refs/heads/main refs/namespaces
git --git-dir=target/cairn-cob/second.git for-each-ref \
  --format='%(refname) %(objectname)' refs/heads/main refs/namespaces
```

Matching refs are bounded local Git observations. They do not prove authentication, authorization, remote publication, peer availability, durability, synchronization, or convergence.

## Second-checkout materialization

From a second Octet root at the recorded base revision with no local active change, first restore the empty lifecycle directory that Git does not track:

```sh
mkdir -p SECOND_ROOT/.cairn/changes

cairn cob status CHANGE \
  --root SECOND_ROOT \
  --git-repository target/cairn-cob/second.git \
  --policy SECOND_ROOT/cairn-policy/generated/cairn-policy.json

cairn cob checkout CHANGE \
  --root SECOND_ROOT \
  --git-repository target/cairn-cob/second.git \
  --policy SECOND_ROOT/cairn-policy/generated/cairn-policy.json
```

Review the checkout destination, artifact hashes, object/state identity, blockers, and non-claims, then repeat with `--execute`. Inspect conflicts separately:

```sh
cairn cob conflicts CHANGE \
  --root SECOND_ROOT \
  --git-repository target/cairn-cob/second.git \
  --policy SECOND_ROOT/cairn-policy/generated/cairn-policy.json
```

## Explicit conflict recovery

A conflict requires distinct authorized actor refs. Embed a delegate's public actor ID during import with `--delegate ACTOR`, then let creator and delegate update separate stores from the same signed root before transporting their actor refs into one inspection store.

Default inspection exposes bounded head metadata without exact content:

```sh
cairn cob conflicts CHANGE \
  --root . \
  --git-repository target/cairn-cob/conflicted.git \
  --policy cairn-policy/generated/cairn-policy.json
```

Reveal one exact value only through an explicit selector copied from the metadata receipt:

```sh
cairn cob conflicts CHANGE \
  --root . \
  --git-repository target/cairn-cob/conflicted.git \
  --policy cairn-policy/generated/cairn-policy.json \
  --show-head tasks=REVISION
```

Ordinary `cob update` must remain blocked while multiple heads are current. Resolution is a separate preview-first mutation:

```sh
cairn cob resolve CHANGE \
  --root . \
  --git-repository target/cairn-cob/conflicted.git \
  --keystore target/cairn-cob/creator-keys \
  --policy cairn-policy/generated/cairn-policy.json \
  --choose-head tasks=REVISION
```

Review the actor, conflict heads, selected artifact, complete action basis, plan hash, expected content root, and non-claims. Repeat with `--execute` only when every current conflicting artifact has an explicit choice. After execution, verify every losing and selected operation remains an ancestor of the resolution operation and reload the refs into a separate store.

Selecting a head does not prove semantic merge correctness or make a losing head invalid. Resolving content also does not silently refresh a stale lifecycle subject root.

The conflict-free pilot under [`fixtures/cairn-cob-pilot/`](../fixtures/cairn-cob-pilot/) and conflict-recovery drill under [`fixtures/cairn-cob-conflict-pilot/`](../fixtures/cairn-cob-conflict-pilot/) bind exact local executions of these workflows. Their receipts are integration evidence only; they do not prove lifecycle gates, evidence truth, network transport, synchronization, distributed convergence, source correctness, whole-program correctness, or release eligibility.
