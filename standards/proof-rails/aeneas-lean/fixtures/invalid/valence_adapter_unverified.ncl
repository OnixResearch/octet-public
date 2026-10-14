# Negative fixture: an adapter export without successful artifact verification.
let fixture = import "../../valence-adapter-fixture.ncl" in
fixture & {
  octet_export = fixture.octet_export & {
    exported_after_verification | force = false,
  },
}
