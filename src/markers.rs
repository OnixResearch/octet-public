//! Matching of configured marker attributes such as `octet::state_machine_enum`.

use rustc_hir as hir;

/// Whether `attrs` carry the configured marker attribute `marker`.
///
/// An attribute matches when its path ends in the marker's last segment, so a legacy
/// `tigerstyle::` or `slotcar::` spelling of the same marker keeps working. Doc comments,
/// attribute arguments, and the file paths in attribute spans never count.
pub(crate) fn attrs_carry_marker(attrs: &[hir::Attribute], marker: &str) -> bool {
    let marker_tail = marker.rsplit("::").next().unwrap_or(marker);
    !marker_tail.is_empty()
        && attrs.iter().any(|attr| {
            attr.is_doc_comment().is_none() && attr.path().last().is_some_and(|segment| segment.as_str() == marker_tail)
        })
}
