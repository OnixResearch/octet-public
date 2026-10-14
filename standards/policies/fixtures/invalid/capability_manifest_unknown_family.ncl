let contracts = import "../../contracts.ncl" in
{
  capability_families = ["telepathy"],
} | { capability_families | Array contracts.CapabilityFamily }
