use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};

use radicle_cob::{change::Entry, Evaluate, Version};
use radicle_oid::Oid;
use serde::Deserialize;
use thiserror::Error;

use super::ReferenceRepository;

const EXPECTED_TYPE_NAME: &str = "xyz.onixresearch.cairn-change";
const EXPECTED_SCHEMA: &str = "cairn.change-collaboration.v1";
const BASE_REVISION_PREFIX: &str = "git_oid:";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceState {
    pub(crate) schema: String,
    pub(crate) change_id: String,
    pub(crate) repository_id: String,
    pub(crate) base_git_oid: String,
    pub(crate) policy_hash: String,
    pub(crate) creator: String,
    pub(crate) delegates: BTreeSet<String>,
    pub(crate) artifacts: BTreeMap<String, String>,
    pub(crate) artifact_heads: BTreeMap<String, BTreeSet<String>>,
    pub(crate) operation_ids: BTreeSet<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ReferenceAction {
    Init {
        state: ReferenceInit,
    },
    SetArtifact {
        artifact: String,
        content: Option<String>,
        basis: BTreeSet<String>,
    },
}

#[derive(Debug, Deserialize)]
struct ReferenceInit {
    schema: String,
    change_id: String,
    repository_id: String,
    base_git_oid: String,
    policy_hash: String,
    #[serde(default)]
    delegates: BTreeSet<String>,
    artifacts: BTreeMap<String, String>,
}

#[derive(Debug, Error)]
pub(crate) enum ReferenceEvaluateError {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("unexpected COB type or version")]
    Manifest,
    #[error("root operation must contain exactly one init action")]
    RootShape,
    #[error("unsupported Cairn schema {0}")]
    Schema(String),
    #[error("root base revision is missing or not related")]
    BaseRevision,
    #[error("creator is duplicated in delegate authority")]
    DuplicateAuthority,
    #[error("concurrent histories are outside this linear reference consumer")]
    Concurrent,
    #[error("init action appeared after the root")]
    Reinitialize,
    #[error("actor {0} is not authorized by the root")]
    Unauthorized(String),
    #[error("artifact {0} does not exist")]
    MissingArtifact(String),
    #[error("artifact {artifact} basis does not match current heads")]
    Basis { artifact: String },
}

impl Evaluate<ReferenceRepository> for ReferenceState {
    type Error = ReferenceEvaluateError;

    fn init(entry: &Entry, _store: &ReferenceRepository) -> Result<Self, Self::Error> {
        validate_manifest(entry)?;
        let actions = decode_actions(entry)?;
        let [ReferenceAction::Init { state }] = actions.as_slice() else {
            return Err(ReferenceEvaluateError::RootShape);
        };
        if state.schema != EXPECTED_SCHEMA {
            return Err(ReferenceEvaluateError::Schema(state.schema.clone()));
        }
        let Some(base) = state.base_git_oid.strip_prefix(BASE_REVISION_PREFIX) else {
            return Err(ReferenceEvaluateError::BaseRevision);
        };
        let base = Oid::from_str(base).map_err(|_| ReferenceEvaluateError::BaseRevision)?;
        if !entry.related.contains(&base) {
            return Err(ReferenceEvaluateError::BaseRevision);
        }
        let creator = entry.author().to_string();
        if state.delegates.contains(&creator) {
            return Err(ReferenceEvaluateError::DuplicateAuthority);
        }
        let operation_id = entry.id().to_string();
        let artifact_heads = state
            .artifacts
            .keys()
            .map(|artifact| (artifact.clone(), BTreeSet::from([operation_id.clone()])))
            .collect();
        Ok(Self {
            schema: state.schema.clone(),
            change_id: state.change_id.clone(),
            repository_id: state.repository_id.clone(),
            base_git_oid: state.base_git_oid.clone(),
            policy_hash: state.policy_hash.clone(),
            creator,
            delegates: state.delegates.clone(),
            artifacts: state.artifacts.clone(),
            artifact_heads,
            operation_ids: BTreeSet::from([operation_id]),
        })
    }

    fn apply<'a, I: Iterator<Item = (&'a Oid, &'a Entry)>>(
        &mut self,
        entry: &Entry,
        mut concurrent: I,
        _store: &ReferenceRepository,
    ) -> Result<(), Self::Error> {
        validate_manifest(entry)?;
        if concurrent.next().is_some() {
            return Err(ReferenceEvaluateError::Concurrent);
        }
        let actor = entry.author().to_string();
        if actor != self.creator && !self.delegates.contains(&actor) {
            return Err(ReferenceEvaluateError::Unauthorized(actor));
        }
        let operation_id = entry.id().to_string();
        for action in decode_actions(entry)? {
            match action {
                ReferenceAction::Init { .. } => return Err(ReferenceEvaluateError::Reinitialize),
                ReferenceAction::SetArtifact {
                    artifact,
                    content,
                    basis,
                } => self.apply_artifact(&operation_id, artifact, content, basis)?,
            }
        }
        self.operation_ids.insert(operation_id);
        Ok(())
    }
}

impl ReferenceState {
    fn apply_artifact(
        &mut self,
        operation_id: &str,
        artifact: String,
        content: Option<String>,
        basis: BTreeSet<String>,
    ) -> Result<(), ReferenceEvaluateError> {
        let heads = self
            .artifact_heads
            .get_mut(&artifact)
            .ok_or_else(|| ReferenceEvaluateError::MissingArtifact(artifact.clone()))?;
        if *heads != basis {
            return Err(ReferenceEvaluateError::Basis { artifact });
        }
        *heads = BTreeSet::from([operation_id.to_string()]);
        match content {
            Some(content) => {
                self.artifacts.insert(artifact, content);
            }
            None => {
                self.artifacts.remove(&artifact);
            }
        }
        Ok(())
    }
}

fn validate_manifest(entry: &Entry) -> Result<(), ReferenceEvaluateError> {
    if entry.manifest.type_name.to_string() == EXPECTED_TYPE_NAME
        && entry.manifest.version == Version::default()
    {
        Ok(())
    } else {
        Err(ReferenceEvaluateError::Manifest)
    }
}

fn decode_actions(entry: &Entry) -> Result<Vec<ReferenceAction>, serde_json::Error> {
    entry
        .contents()
        .iter()
        .map(|content| serde_json::from_slice(content))
        .collect()
}
