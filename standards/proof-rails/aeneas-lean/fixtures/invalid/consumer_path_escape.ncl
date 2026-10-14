let contracts = import "../../consumer-contracts.ncl" in
let profile = import "../../consumer-profile.ncl" in
let source = std.array.first profile.source.paths in
(profile & { source.paths | force = [source & { path | force = "../outside.rs" }] })
| contracts.AeneasConsumerProfile
| contracts.AeneasConsumerProfileRules
