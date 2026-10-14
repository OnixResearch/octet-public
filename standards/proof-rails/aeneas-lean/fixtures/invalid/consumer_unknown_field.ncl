let contracts = import "../../consumer-contracts.ncl" in
let profile = import "../../consumer-profile.ncl" in
(profile & { ambient_provider | force = "host" })
| contracts.AeneasConsumerProfile
| contracts.AeneasConsumerProfileRules
