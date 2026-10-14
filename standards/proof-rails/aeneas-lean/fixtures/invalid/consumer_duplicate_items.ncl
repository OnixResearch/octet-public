let contracts = import "../../consumer-contracts.ncl" in
let profile = import "../../consumer-profile.ncl" in
let item = std.array.first profile.source.selected_items in
(profile & { source.selected_items | force = [item, item] })
| contracts.AeneasConsumerProfile
| contracts.AeneasConsumerProfileRules
