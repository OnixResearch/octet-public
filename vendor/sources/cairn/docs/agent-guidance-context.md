# Agent guidance context

Cairn can load declared project facts and advisory guidance for local agent work.

Cairn does not scan the repository for prompt files. It reads only entries in the checked project configuration.

## Authority classes

Each result entry has one authority class.

| Class | Source | Meaning |
|---|---|---|
| `policy_constraint` | Generated policy | A command-enforced artifact constraint. |
| `project_fact` | A declared local file | Project information for authoring. |
| `advisory_guidance` | Declared inline text | Advice that Cairn does not enforce. |

The output order is always policy constraints, project facts, and advisory guidance.

Project configuration cannot create `policy_constraint` entries. It cannot change command authorization, paths, gates, profiles, mutations, evidence, or release decisions.

Validation, gate, sync, archive, evidence, and release commands ignore advisory prose when they make authority decisions.

## Project declaration

Add the fields to `.cairn/project.ncl`.

```nickel
{
  schema = "cairn.lifecycle-store-project-source.v1",
  lifecycle_store = null,
  references = [],

  context_refs = [
    {
      id = "project-overview",
      source_root = "project",
      path = "docs/project-overview.md",
      required = true,
      priority = 10,
      authority = "project_fact",
      profiles = ["assurance"],
      artifacts = ["tasks"],
      operations = ["context", "workflow-instructions"],
    },
  ],

  artifact_guidance = [
    {
      id = "task-evidence",
      text = "Keep task evidence local and bounded.",
      priority = 20,
      authority = "advisory_guidance",
      profiles = ["assurance"],
      artifacts = ["tasks"],
    },
  ],

  operation_guidance = [
    {
      id = "context-use",
      text = "Treat project guidance as advisory input.",
      priority = 30,
      authority = "advisory_guidance",
      operations = ["context"],
    },
  ],

  non_claims = [
    "a project pointer selects one declared store but does not grant lifecycle authority",
    "a direct reference provides read-only discovery and does not satisfy dependencies, gates, evidence, or requirements",
    "reference indexing does not prove truth, freshness, ownership, implementation, or satisfaction",
    "project configuration does not authorize transitive reference traversal or transport",
    "project facts and advisory guidance do not override generated policy or command authorization",
    "project context checks do not prove truth, secrecy, semantic consistency, agent compliance, or release eligibility",
  ],
}
```

Run the checked export after each source change.

```console
cairn config export --kind project --root . --execute
cairn config export --kind project --root . --check
```

Normal context and workflow commands read `.cairn/project.json`. They do not evaluate Nickel.

Cairn rejects a stale source and projection pair.

## Context references

A context reference selects one UTF-8 text file.

`source_root` accepts these values:

- `project`: Resolve the path under the observed project root.
- `lifecycle`: Resolve the path under the selected lifecycle root.

The path must be a safe relative path. Cairn rejects these values:

- absolute paths
- parent traversal
- symlinks
- globs
- URLs
- environment expansion
- command syntax
- invalid UTF-8
- text larger than the policy bound

Cairn uses a capability root for each file read. It does not use an ambient fallback.

A missing required file makes the selected result invalid. A missing optional file remains visible and does not invalidate the result.

## Selectors

Entries can select a workflow profile, artifact, or operation.

An empty selector list matches all admitted requests for that selector class.

The generated policy defines all known selector values. Cairn rejects unknown declaration and request values before it reads context files.

Cairn removes repeated selectors. It then uses stable ordering by authority, priority, source root, path, and ID.

## CLI use

Inspect context for one authoring request.

```console
cairn context --root . --profile assurance --artifact tasks
```

Select an explicit policy-known operation when necessary.

```console
cairn context --root . --operation context
```

Get artifact instructions with selected context.

```console
cairn workflow instructions tasks --change my-change --root .
```

A guidance result includes these values:

- the admitted request and policy bounds
- local entries with text and relative paths
- portable entries without raw text
- source byte lengths and BLAKE3 identities
- authority classes and source status
- stable diagnostics
- a full local result hash
- a checkout-independent portable identity
- policy and project identities
- bounded non-claims

These commands are read-only. They do not edit files, run declared commands, contact a network, or satisfy a gate.

## MCP use

Use `cairn_context` for the same context selection through MCP.

The tool accepts `root`, `policy`, `profile`, `artifact`, and `operation`. It rejects unknown fields and wrong types.

Use `cairn_workflow_instructions` for artifact instructions. Both tools use the same pure selection and result core as the CLI.

MCP framing differs from CLI framing. Selected entries, diagnostics, authority classes, and guidance identities remain equivalent.

## Local and portable evidence

The local result can contain raw declared text. Its `local_result_hash` binds that text and the observed source facts.

The `portable_evidence` value omits raw text and absolute paths. It keeps relative paths, source BLAKE3 identities, selectors, authority classes, policy, and project identity.

Equivalent checked inputs produce the same portable identity under different safe checkout paths.

Portable admission rejects a value that adds `text` or `absolute_path` fields.

## Privacy duties

Declare only text that local agents may read.

Cairn checks paths, encoding, size, selection, and portable omission. These checks do not prove that text is secret-free or safe to disclose.

Cairn does not read undeclared files. It does not expand a declared file into a directory search.

## Compatibility

Projects can omit all three guidance collections. Existing project projections without these fields remain readable as empty guidance declarations.

When guidance is empty, Cairn does not add a guidance object to existing context or workflow receipts.

## Claim boundary

A valid result proves bounded declaration admission, selected observed bytes, authority labels, identities, and deterministic ordering.

It does not prove truth, secrecy, semantic consistency, agent compliance, policy satisfaction, implementation correctness, or release eligibility.
