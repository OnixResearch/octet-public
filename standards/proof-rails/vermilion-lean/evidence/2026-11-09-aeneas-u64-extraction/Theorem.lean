import Llbc

open Aeneas Aeneas.Std Result

/-- The generated Aeneas model preserves the u64 checked-increment contract:
under the precondition, the model returns `some (value + 1)`. -/
theorem increment_checked_contract (value : Std.U64) (h : value < core.num.U64.MAX) :
    u64_pilot.increment_checked value =
      (do
        let i ← value + 1#u64
        ok (some i)) := by
  simp [u64_pilot.increment_checked, h]

/-- At `u64::MAX` the generated model returns `none`. -/
theorem increment_checked_at_max :
    u64_pilot.increment_checked core.num.U64.MAX = ok none := by
  simp [u64_pilot.increment_checked]
