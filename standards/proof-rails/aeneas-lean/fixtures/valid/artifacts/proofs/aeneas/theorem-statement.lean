import Llbc

/-- Named Octet-owned theorem over the generated Aeneas model. -/
theorem safe_add_matches_model : real_fixture.safe_add 1#u32 2#u32 = .ok 3#u32 := by
  rfl
