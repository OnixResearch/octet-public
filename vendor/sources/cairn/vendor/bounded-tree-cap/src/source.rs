//! Bounded no-follow observation through a caller-opened source capability.

mod entry;
mod traverse;

use crate::prelude::{
    Dir, OperationKind, ShellError, ShellErrorKind, SymlinkPolicy, TreeLimits, TreePlan,
    TreeRequest, plan_tree,
};

/// A complete accepted plan bound to a cloned source capability.
#[derive(Debug)]
pub struct PreparedTree {
    pub(crate) source: Dir,
    pub(crate) plan: TreePlan,
}

impl PreparedTree {
    /// Returns the complete pure plan.
    #[must_use]
    pub const fn plan(&self) -> &TreePlan {
        &self.plan
    }
}

/// Observes, hashes, and plans one tree beneath a caller-opened capability.
///
/// # Errors
///
/// Returns a typed limit, I/O, source-change, or pure-plan failure.
// r[impl bounded_tree.shell.capability_io]
// r[impl bounded_tree.shell.bounded_observation]
pub fn prepare(
    source: &Dir,
    limits: TreeLimits,
    symlink_policy: SymlinkPolicy,
) -> Result<PreparedTree, ShellError> {
    let source = source
        .try_clone()
        .map_err(|error| ShellError::io(OperationKind::CloneCapability, &[], error))?;
    let observations = traverse::observe_tree(&source, limits)?;
    let plan = plan_tree(TreeRequest {
        observations,
        limits,
        symlink_policy,
    })
    .map_err(|error| ShellError::new(ShellErrorKind::PlanRejected(error), Vec::new()))?;
    Ok(PreparedTree { source, plan })
}
