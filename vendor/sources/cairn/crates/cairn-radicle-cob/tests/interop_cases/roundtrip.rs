use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fs,
    str::FromStr,
};

use cairn_core::{
    apply_change_cob_operation, change_cob_content_root, change_cob_import_content_root,
    ChangeArtifactId, ChangeBaseRevision, ChangeCobAction, ChangeCobInit, ChangeCobOperation,
    CHANGE_COLLABORATION_SCHEMA_V1,
};
use onix_cairn_radicle_cob::{create_change, inspect_change, update_change, TYPENAME};
use radicle_cob::ObjectId;

use crate::interop_support::{
    initialize_keystore, initialize_repository, provenance, remove_directory, ExternalFixture,
    ReferenceRepository, ReferenceState,
};

const EXTERNAL_UPDATED_PROPOSAL: &str = "# Updated by Cairn delegate\n";
const CAIRN_CHANGE_ID: &str = "cairn-produced-for-reference-consumer";
const CAIRN_REPOSITORY_ID: &str = "workspace:cairn-reference-consumer";
const POLICY_HASH: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const CAIRN_INITIAL_PROPOSAL: &str = "# Cairn initial proposal\n";
const CAIRN_UPDATED_PROPOSAL: &str = "# Cairn output consumed directly\n";
const PREVIEW_EXTERNAL_UPDATE_ID: &str = "preview:external-update";
const PREVIEW_CAIRN_UPDATE_ID: &str = "preview:cairn-update";
const CREATOR_SEED_BYTE: u8 = 51;
const DELEGATE_SEED_BYTE: u8 = 52;
const EXPECTED_LINEAR_OPERATION_COUNT: usize = 2;

// r[impl cairn.cob_interop.inward]
// r[verify cairn.cob_interop.inward.update]
#[test]
fn cairn_loads_and_updates_external_radicle_storage_fixture() -> Result<(), Box<dyn Error>> {
    let provenance = provenance()?;
    let fixture = ExternalFixture::new("external-inward", &provenance)?;
    let fixture_repository = git2::Repository::open(&fixture.repository)?;
    assert_eq!(fixture_repository.remotes()?.len(), 0);
    assert_eq!(
        fixture_repository
            .find_reference(&provenance.fixture.reference)?
            .target()
            .map(|target| target.to_string()),
        Some(provenance.fixture.operation_id.clone())
    );
    drop(fixture_repository);
    let initial = inspect_change(&fixture.repository, &provenance.fixture.change_id)?;
    assert_eq!(initial.object_id, provenance.fixture.object_id);
    assert_eq!(initial.state.creator, provenance.fixture.actor);
    assert_eq!(
        initial.state.repository_id,
        provenance.fixture.repository_id
    );
    assert_eq!(
        initial.state.base_git_oid,
        format!("git_oid:{}", provenance.fixture.base_oid)
    );
    assert_eq!(
        initial.state.delegates,
        BTreeSet::from([provenance.fixture.delegate.clone()])
    );
    let initial_heads = initial.state.artifacts[&ChangeArtifactId::proposal()].heads();
    assert_eq!(initial_heads.len(), 1);
    assert_eq!(
        initial_heads[0].1.value.as_deref(),
        Some(provenance.fixture.proposal.as_str())
    );
    assert!(initial
        .state
        .operations
        .contains(&provenance.fixture.operation_id));

    let basis = initial.state.artifacts[&ChangeArtifactId::proposal()].head_ids();
    let actions = vec![ChangeCobAction::SetArtifact {
        artifact: ChangeArtifactId::proposal(),
        content: Some(EXTERNAL_UPDATED_PROPOSAL.to_string()),
        basis,
    }];
    let mut expected = initial.state.clone();
    apply_change_cob_operation(
        &mut expected,
        ChangeCobOperation {
            id: PREVIEW_EXTERNAL_UPDATE_ID.to_string(),
            actor: fixture.delegate.to_string(),
            is_delegate: true,
            actions: actions.clone(),
        },
    )?;
    let expected_root = change_cob_content_root(&expected)?;
    let updated = update_change(
        &fixture.repository,
        &fixture.delegate_keystore,
        &initial.object_id,
        &provenance.fixture.change_id,
        &fixture.delegate.to_string(),
        &actions,
        &expected_root,
    )?;
    let reloaded = inspect_change(&fixture.repository, &provenance.fixture.change_id)?;
    assert_eq!(change_cob_content_root(&reloaded.state)?, expected_root);
    assert!(reloaded
        .state
        .operations
        .contains(&provenance.fixture.operation_id));
    assert!(reloaded.state.operations.contains(&updated.operation_id));
    let reloaded_heads = reloaded.state.artifacts[&ChangeArtifactId::proposal()].heads();
    assert_eq!(reloaded_heads.len(), 1);
    assert_eq!(
        reloaded_heads[0].1.value.as_deref(),
        Some(EXTERNAL_UPDATED_PROPOSAL)
    );
    let fixture_repository = git2::Repository::open(&fixture.repository)?;
    assert_eq!(fixture_repository.remotes()?.len(), 0);
    assert!(fixture_repository
        .find_commit(provenance.fixture.operation_id.parse()?)
        .is_ok());
    Ok(())
}

// r[impl cairn.cob_interop.outward]
// r[verify cairn.cob_interop.outward.roundtrip]
#[test]
fn direct_reference_consumer_evaluates_cairn_output() -> Result<(), Box<dyn Error>> {
    let root = std::env::temp_dir().join(format!(
        "cairn-cob-reference-consumer-{}",
        std::process::id()
    ));
    remove_directory(&root)?;
    fs::create_dir_all(&root)?;
    let repository_path = root.join("repository");
    let repository = initialize_repository(&repository_path)?;
    let creator_keys = root.join("creator-keys");
    let delegate_keys = root.join("delegate-keys");
    let creator = initialize_keystore(&creator_keys, CREATOR_SEED_BYTE)?;
    let delegate = initialize_keystore(&delegate_keys, DELEGATE_SEED_BYTE)?;
    let base_oid = repository.head()?.target().ok_or("repository HEAD")?;
    drop(repository);
    let init = ChangeCobInit {
        schema: CHANGE_COLLABORATION_SCHEMA_V1.to_string(),
        change_id: CAIRN_CHANGE_ID.to_string(),
        repository_id: CAIRN_REPOSITORY_ID.to_string(),
        base_git_oid: ChangeBaseRevision::new("git_oid", base_oid.to_string())?.encode()?,
        policy_hash: POLICY_HASH.to_string(),
        delegates: BTreeSet::from([delegate.to_string()]),
        artifacts: BTreeMap::from([(
            ChangeArtifactId::proposal(),
            CAIRN_INITIAL_PROPOSAL.to_string(),
        )]),
    };
    let expected_root = change_cob_import_content_root(&init, &creator.to_string())?;
    let created = create_change(&repository_path, &creator_keys, init, &expected_root)?;
    let initial = inspect_change(&repository_path, CAIRN_CHANGE_ID)?;
    let actions = vec![ChangeCobAction::SetArtifact {
        artifact: ChangeArtifactId::proposal(),
        content: Some(CAIRN_UPDATED_PROPOSAL.to_string()),
        basis: BTreeSet::from([created.operation_id.clone()]),
    }];
    let mut expected = initial.state;
    apply_change_cob_operation(
        &mut expected,
        ChangeCobOperation {
            id: PREVIEW_CAIRN_UPDATE_ID.to_string(),
            actor: delegate.to_string(),
            is_delegate: true,
            actions: actions.clone(),
        },
    )?;
    let updated = update_change(
        &repository_path,
        &delegate_keys,
        &created.object_id,
        CAIRN_CHANGE_ID,
        &delegate.to_string(),
        &actions,
        &change_cob_content_root(&expected)?,
    )?;

    let reference_repository = ReferenceRepository::open(&repository_path)?;
    let object_id = ObjectId::from_str(&created.object_id)?;
    let loaded =
        radicle_cob::get::<ReferenceState, _>(&reference_repository, &TYPENAME, &object_id)?
            .ok_or("reference consumer did not find Cairn object")?;
    assert_eq!(loaded.object.change_id, CAIRN_CHANGE_ID);
    assert_eq!(loaded.object.repository_id, CAIRN_REPOSITORY_ID);
    assert_eq!(loaded.object.schema, CHANGE_COLLABORATION_SCHEMA_V1);
    assert_eq!(loaded.object.base_git_oid, format!("git_oid:{base_oid}"));
    assert_eq!(loaded.object.policy_hash, POLICY_HASH);
    assert_eq!(loaded.object.creator, creator.to_string());
    assert_eq!(
        loaded.object.delegates,
        BTreeSet::from([delegate.to_string()])
    );
    assert_eq!(
        loaded.object.artifacts[ChangeArtifactId::proposal().as_str()],
        CAIRN_UPDATED_PROPOSAL
    );
    assert_eq!(
        loaded.object.operation_ids,
        BTreeSet::from([created.operation_id, updated.operation_id])
    );
    assert_eq!(
        loaded.object.operation_ids.len(),
        EXPECTED_LINEAR_OPERATION_COUNT
    );
    remove_directory(&root)?;
    Ok(())
}
