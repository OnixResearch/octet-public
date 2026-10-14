pub(crate) mod evidence_codebase_index {
    pub(crate) use valence_core::evidence_codebase_index::EvidenceArtifactInput;
    pub(crate) use valence_core::evidence_codebase_index::EvidenceCodebaseIndex;
    pub(crate) use valence_core::evidence_codebase_index::EvidenceIndexBuildMode;
    pub(crate) use valence_core::evidence_codebase_index::EvidenceIndexIssueSeverity;
    pub(crate) use valence_core::evidence_codebase_index::build_evidence_codebase_index;
    pub(crate) use valence_core::evidence_codebase_index::query_evidence_index;
    pub(crate) use valence_core::evidence_codebase_index::render_evidence_index_html;
}

pub(crate) mod evidence_index {
    pub(crate) use valence_core::evidence_index::EvidenceIndexInput;
    #[cfg(test)]
    pub(crate) use valence_core::evidence_index::check_page_text;
    pub(crate) use valence_core::evidence_index::check_page_text_with_expected;
    pub(crate) use valence_core::evidence_index::render_valence_evidence_index;
}

pub(crate) mod function_object {
    pub(crate) use valence_core::function_object::*;
}
