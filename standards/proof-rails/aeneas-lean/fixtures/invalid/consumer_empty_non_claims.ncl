let contracts = import "../../consumer-contracts.ncl" in
let profile = import "../../consumer-profile.ncl" in
(profile & { non_claims | force = [] })
| contracts.AeneasConsumerProfile
| contracts.AeneasConsumerProfileRules
