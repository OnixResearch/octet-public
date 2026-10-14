# Tiger Style Standards Fixtures

Fixtures exercise `octet-standards` behavior independently from production source.

- `suppressions/compliant/` contains source that should pass while reporting active narrow suppressions.
- `suppressions/noncompliant/` contains source that should fail with stable suppression diagnostics.
- `architecture/compliant/` contains FCIS-shaped source that should pass architecture checks.
- `architecture/noncompliant/` contains missing-`no_std`, forbidden-runtime-dependency, std-shell, thick-root, and external-domain-leakage failures.
- `architecture/advisory/` contains public API budget fixtures that should report warnings without failing the command.
- `embedded/compliant/` contains bounded queue/channel source that should pass resource checks.
- `embedded/noncompliant/` contains unbounded queue/channel source that should fail resource checks.

The flake check `standards-fixtures` validates these shapes with exported policy registry JSON.
