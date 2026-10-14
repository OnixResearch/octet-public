let Contracts = import "../../contracts/generated.ncl" in
let Decision = import "../../../mirai/decision.json" in
Decision | Contracts.MiraiPilotDecisionV1
