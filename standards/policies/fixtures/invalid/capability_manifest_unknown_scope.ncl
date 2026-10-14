let contracts = import "../../contracts.ncl" in
{
  capability_scope_classes = ["liminal"],
} | { capability_scope_classes | Array contracts.CapabilityScopeClass }
