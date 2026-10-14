let contracts = import "../../consumer-contracts.ncl" in
let profile = import "../../consumer-profile.ncl" in
(profile & { provider.profile_id | force = "ambient-provider" })
| contracts.AeneasConsumerProfile
| contracts.AeneasConsumerProfileRules
