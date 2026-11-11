use std::{error::Error, num::NonZeroUsize};

use onix_cairn_radicle_cob::{
    inspect_change, CairnChangeCob, CobError, CobRepository, CobStoreError, TYPENAME,
};
use radicle_cob::{change, Evaluate, Manifest, Version};

use crate::interop_support::{
    provenance, verify_fixture_identity, ExternalFixture, BUNDLE_BYTES, PRODUCER_LOCK_BYTES,
    PRODUCER_MANIFEST_BYTES, PRODUCER_SOURCE_BYTES, REFERENCE_EVALUATOR_SOURCE,
    REFERENCE_STORAGE_SOURCE,
};

const PROVENANCE_SCHEMA: &str = "cairn.radicle-cob-interoperability-provenance.v1";
const REFERENCE_RADICLE_VERSION: &str = "0.24.0";
const REFERENCE_RADICLE_COB_VERSION: &str = "0.20.0";
const REFERENCE_RADICLE_CRYPTO_VERSION: &str = "0.17.0";
const REFERENCE_RADICLE_OID_VERSION: &str = "0.2.0";
const REFERENCE_GIT2_VERSION: &str = "0.20.4";
const REFERENCE_TYPE_NAME: &str = "xyz.onixresearch.cairn-change";
const UMBRELLA_RADICLE_PACKAGE: &str = "radicle";
const RADICLE_COB_PACKAGE: &str = "radicle-cob";
const RADICLE_CRYPTO_PACKAGE: &str = "radicle-crypto";
const RADICLE_OID_PACKAGE: &str = "radicle-oid";
const GIT2_PACKAGE: &str = "git2";
const RADICLE_NODE_PACKAGE: &str = "radicle-node";
const UNSUPPORTED_VERSION_NUMBER: usize = 2;
const FORBIDDEN_CONSUMER_DEPENDENCIES: &[&str] = &[
    "cairn_core",
    "CobRepository",
    "CairnChangeCob",
    "inspect_change",
    "apply_change_cob_operation",
];

// r[impl cairn.cob_interop.fixture]
// r[verify cairn.cob_interop.fixture.current]
// r[verify cairn.cob_interop.fixture.external]
// r[impl cairn.cob_interop.boundary]
// r[verify cairn.cob_interop.boundary.offline]
// r[impl cairn.cob_interop.validation]
// r[verify cairn.cob_interop.validation.release]
#[test]
fn provenance_binds_standalone_radicle_producer() -> Result<(), Box<dyn Error>> {
    let provenance = provenance()?;
    verify_fixture_identity(
        BUNDLE_BYTES,
        PRODUCER_SOURCE_BYTES,
        PRODUCER_MANIFEST_BYTES,
        PRODUCER_LOCK_BYTES,
        &provenance,
    )?;
    assert_eq!(provenance.schema, PROVENANCE_SCHEMA);
    assert_eq!(
        provenance.producer.radicle_version,
        REFERENCE_RADICLE_VERSION
    );
    assert_eq!(
        provenance.producer.radicle_cob_version,
        REFERENCE_RADICLE_COB_VERSION
    );
    assert_eq!(
        provenance.producer.radicle_crypto_version,
        REFERENCE_RADICLE_CRYPTO_VERSION
    );
    assert_eq!(
        provenance.producer.radicle_oid_version,
        REFERENCE_RADICLE_OID_VERSION
    );
    assert_eq!(provenance.producer.git2_version, REFERENCE_GIT2_VERSION);
    assert_eq!(provenance.fixture.type_name, REFERENCE_TYPE_NAME);
    assert!(provenance
        .producer
        .implementation
        .contains("radicle::storage::git::Repository"));
    assert!(provenance.producer.generation_command.contains("--locked"));
    assert!(provenance.producer.bundle_command.contains("bundle create"));
    assert!(provenance
        .non_claims
        .iter()
        .all(|non_claim| !non_claim.trim().is_empty()));
    for (package, version) in [
        (UMBRELLA_RADICLE_PACKAGE, REFERENCE_RADICLE_VERSION),
        (RADICLE_COB_PACKAGE, REFERENCE_RADICLE_COB_VERSION),
        (RADICLE_CRYPTO_PACKAGE, REFERENCE_RADICLE_CRYPTO_VERSION),
        (RADICLE_OID_PACKAGE, REFERENCE_RADICLE_OID_VERSION),
        (GIT2_PACKAGE, REFERENCE_GIT2_VERSION),
    ] {
        assert!(lockfile_has_package_version(
            PRODUCER_LOCK_BYTES,
            package,
            version
        ));
    }
    assert!(!lockfile_has_package(
        PRODUCER_LOCK_BYTES,
        RADICLE_NODE_PACKAGE
    ));
    assert!(String::from_utf8_lossy(PRODUCER_MANIFEST_BYTES).contains("[workspace]"));
    for source in [REFERENCE_STORAGE_SOURCE, REFERENCE_EVALUATOR_SOURCE] {
        assert!(source.contains("radicle_cob"));
        for forbidden in FORBIDDEN_CONSUMER_DEPENDENCIES {
            assert!(!source.contains(forbidden));
        }
    }
    for path in [
        &provenance.fixture.bundle_path,
        &provenance.fixture.producer_source_path,
        &provenance.fixture.producer_manifest_path,
        &provenance.fixture.producer_lock_path,
    ] {
        assert!(path.starts_with("fixtures/radicle-cob-interoperability/"));
    }
    Ok(())
}

// r[impl cairn.cob_interop.negative]
// r[verify cairn.cob_interop.negative.hash]
// r[verify cairn.cob_interop.negative.unsigned]
// r[verify cairn.cob_interop.negative.version]
#[test]
fn tampered_missing_unsigned_and_version_fixtures_fail_closed() -> Result<(), Box<dyn Error>> {
    let provenance = provenance()?;
    let mut tampered = BUNDLE_BYTES.to_vec();
    let first = tampered.first_mut().ok_or("fixture bundle is empty")?;
    *first ^= u8::MAX;
    assert!(verify_fixture_identity(
        &tampered,
        PRODUCER_SOURCE_BYTES,
        PRODUCER_MANIFEST_BYTES,
        PRODUCER_LOCK_BYTES,
        &provenance,
    )
    .is_err());

    let unsigned = ExternalFixture::new("unsigned", &provenance)?;
    let repository = git2::Repository::open(&unsigned.repository)?;
    repository.reference(
        &provenance.fixture.reference,
        provenance.fixture.base_oid.parse()?,
        true,
        "inject unsigned fixture history",
    )?;
    assert!(matches!(
        inspect_change(&unsigned.repository, &provenance.fixture.change_id),
        Err(CobError::Store(CobStoreError::InvalidCobChange { .. }))
    ));

    let missing = ExternalFixture::new("missing-ref", &provenance)?;
    git2::Repository::open(&missing.repository)?
        .find_reference(&provenance.fixture.reference)?
        .delete()?;
    assert!(matches!(
        inspect_change(&missing.repository, &provenance.fixture.change_id),
        Err(CobError::ChangeMissing { .. })
    ));

    let incompatible = ExternalFixture::new("incompatible-version", &provenance)?;
    let storage = CobRepository::open(&incompatible.repository)?;
    let operation = provenance
        .fixture
        .operation_id
        .parse::<radicle_oid::Oid>()?;
    let mut entry = <CobRepository as change::Storage>::load(&storage, operation)?;
    let version = NonZeroUsize::new(UNSUPPORTED_VERSION_NUMBER)
        .ok_or("unsupported fixture version must be non-zero")?;
    entry.manifest = Manifest::new(TYPENAME.clone(), Version::from(version));
    assert!(matches!(
        CairnChangeCob::init(&entry, &storage),
        Err(CobError::VersionMismatch {
            actual: UNSUPPORTED_VERSION_NUMBER,
            ..
        })
    ));
    Ok(())
}

fn lockfile_has_package(lockfile: &[u8], name: &str) -> bool {
    String::from_utf8_lossy(lockfile).contains(&format!("\nname = \"{name}\"\n"))
}

fn lockfile_has_package_version(lockfile: &[u8], name: &str, version: &str) -> bool {
    String::from_utf8_lossy(lockfile)
        .contains(&format!("\nname = \"{name}\"\nversion = \"{version}\"\n"))
}
