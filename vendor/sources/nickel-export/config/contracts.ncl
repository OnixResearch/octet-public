let RepositoryContractValue = {
  name | String,
  repository | String,
  license | String,
  packages | Array String,
  schemas | Array String,
  evaluator | {
    identity | String,
    version | String,
    import_path_policy | String,
  },
  release_checks | Array String,
  non_claims | Array String,
} in
{
  RepositoryContract = RepositoryContractValue,
}
