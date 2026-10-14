let profile = { enabled = false } in
if profile.enabled then profile else error "disabled-profile"
