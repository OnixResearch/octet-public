let contracts = import "../../consumer-contracts.ncl" in
let profile = import "../../consumer-profile.ncl" in
let theorem = std.array.first profile.theorems in
(profile & { theorems | force = [theorem & { requirement_id | force = "" }] })
| contracts.AeneasConsumerProfile
| contracts.AeneasConsumerProfileRules
