let policy = import "pilot.ncl" in
{
  schema_version = "octet-mirai-semantic-pilot-corpus/v1",
  fixtures = policy.profile.corpus,
}
