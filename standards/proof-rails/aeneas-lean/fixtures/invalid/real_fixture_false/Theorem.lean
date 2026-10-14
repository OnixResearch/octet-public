import Llbc

/-- Negative fixture: the generated model cannot prove the false result. -/
theorem false_safe_add : real_fixture.safe_add 1#u32 2#u32 = .ok 4#u32 := by
  rfl
