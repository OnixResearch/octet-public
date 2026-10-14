# Valid generated artifact integrity manifest fixture.
let contracts = import "../../contracts.ncl" in

{
  schema_version = "octet-artifact-manifest/v1",
  generated_from = "standards/policies/registry.ncl",
  registry_schema_version = "1",
  artifacts = [
    {
      path = "octet-check-config.toml",
      blake3 = "1111111111111111111111111111111111111111111111111111111111111111",
      bytes = 42,
    },
    {
      path = "octet-policies.json",
      blake3 = "2222222222222222222222222222222222222222222222222222222222222222",
      bytes = 84,
    },
  ],
} | contracts.ArtifactManifest
