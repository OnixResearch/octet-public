let shared = import "_shared-import.ncl" in
{ evaluator = shared.evaluator, imported = true }
