use std::{collections::BTreeMap, fs, path::Path, str::FromStr};

use git_ref_format_core::{RefStr, RefString};
use radicle_cob::{
    change,
    object::{self, Reference},
    ObjectId, Store, TypeName,
};
use radicle_crypto::PublicKey;
use radicle_oid::Oid;
use thiserror::Error;

const COB_REF_PREFIX: &str = "refs/namespaces";
const COB_REF_MIDDLE: &str = "refs/cobs";
const REF_LOG_MESSAGE: &str = "Update Cairn collaborative object";
const REF_ROLLBACK_LOG_MESSAGE: &str = "Rollback failed Cairn COB postcondition";
const NAMESPACE_COMPONENT_INDEX: usize = 2;

#[derive(Debug, Error)]
pub enum CobStoreError {
    #[error("Git repository path is a symlink: {path}")]
    SymlinkRepository { path: String },
    #[error("failed to inspect Git repository path: {0}")]
    RepositoryIo(#[from] std::io::Error),
    #[error("failed to access Git COB storage: {0}")]
    Git(#[from] git2::Error),
    #[error("invalid COB reference format: {0}")]
    RefFormat(#[from] git_ref_format_core::Error),
    #[error("invalid COB reference target: {0}")]
    Reference(#[from] radicle_cob::object::storage::convert::Error),
    #[error("COB reference is not UTF-8")]
    NonUtf8Reference,
    #[error("reference does not identify a Cairn COB: {reference}")]
    InvalidCobReference { reference: String },
    #[error("reference contains an invalid signed COB change: {reference}: {reason}")]
    InvalidCobChange { reference: String, reason: String },
    #[error("COB reference changed outside the new operation ancestry: {reference}")]
    ReferenceDrift { reference: String },
    #[error("explicit Git repository lacks non-empty local config {field}")]
    MissingLocalCommitIdentity { field: &'static str },
}

pub struct CobRepository {
    raw: git2::Repository,
}

impl CobRepository {
    pub fn open(path: &Path) -> Result<Self, CobStoreError> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Err(CobStoreError::SymlinkRepository {
                path: path.display().to_string(),
            });
        }
        Ok(Self {
            raw: git2::Repository::open(path)?,
        })
    }

    pub fn raw(&self) -> &git2::Repository {
        &self.raw
    }

    pub fn head_oid(&self) -> Result<Oid, CobStoreError> {
        Ok(self.raw.head()?.peel_to_commit()?.id().into())
    }

    pub fn object_ids(&self, type_name: &TypeName) -> Result<Vec<ObjectId>, CobStoreError> {
        let mut ids = <Self as object::Storage>::types(self, type_name)?
            .into_keys()
            .collect::<Vec<_>>();
        ids.sort();
        Ok(ids)
    }

    pub fn validate_local_commit_identity(&self) -> Result<(), CobStoreError> {
        const REQUIRED_LOCAL_FIELDS: &[&str] = &["user.name", "user.email"];

        let config = self.raw.config()?.open_level(git2::ConfigLevel::Local)?;
        for &field in REQUIRED_LOCAL_FIELDS {
            match config.get_string(field) {
                Ok(value) if !value.trim().is_empty() => {}
                Ok(_) => return Err(CobStoreError::MissingLocalCommitIdentity { field }),
                Err(error) if error.code() == git2::ErrorCode::NotFound => {
                    return Err(CobStoreError::MissingLocalCommitIdentity { field });
                }
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }

    pub fn namespace_ref_target(
        &self,
        namespace: &PublicKey,
        type_name: &TypeName,
        object_id: &ObjectId,
    ) -> Result<Option<Oid>, CobStoreError> {
        let name = cob_ref_name(namespace, type_name, object_id)?;
        match self.raw.find_reference(&name) {
            Ok(reference) => reference
                .target()
                .map(Into::into)
                .ok_or(CobStoreError::InvalidCobReference { reference: name })
                .map(Some),
            Err(error) if error.code() == git2::ErrorCode::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    pub fn restore_namespace_ref(
        &self,
        namespace: &PublicKey,
        type_name: &TypeName,
        object_id: &ObjectId,
        expected_current: Oid,
        target: Option<Oid>,
    ) -> Result<(), CobStoreError> {
        let name = cob_ref_name(namespace, type_name, object_id)?;
        match target {
            Some(target) => {
                self.raw.reference_matching(
                    &name,
                    target.into(),
                    true,
                    expected_current.into(),
                    REF_ROLLBACK_LOG_MESSAGE,
                )?;
            }
            None => match self.raw.find_reference(&name) {
                Ok(mut reference) if reference.target() == Some(expected_current.into()) => {
                    reference.delete()?;
                }
                Ok(_) => return Err(CobStoreError::ReferenceDrift { reference: name }),
                Err(error) if error.code() == git2::ErrorCode::NotFound => {}
                Err(error) => return Err(error.into()),
            },
        }
        Ok(())
    }

    fn validate_reference_change(
        &self,
        reference: &git2::Reference<'_>,
        type_name: &TypeName,
    ) -> Result<(), CobStoreError> {
        let name = reference
            .name()
            .ok_or(CobStoreError::NonUtf8Reference)?
            .to_string();
        let target = reference
            .target()
            .ok_or_else(|| CobStoreError::InvalidCobReference {
                reference: name.clone(),
            })?;
        let entry = <git2::Repository as change::Storage>::load(&self.raw, target.into()).map_err(
            |error| CobStoreError::InvalidCobChange {
                reference: name.clone(),
                reason: error.to_string(),
            },
        )?;
        let manifest =
            <git2::Repository as change::Storage>::manifest_of(&self.raw, &target.into()).map_err(
                |error| CobStoreError::InvalidCobChange {
                    reference: name.clone(),
                    reason: error.to_string(),
                },
            )?;
        let namespace = namespace_from_ref(&name)?;
        if manifest.type_name != *type_name || entry.author() != &namespace {
            return Err(CobStoreError::InvalidCobChange {
                reference: name,
                reason: "manifest type or signed author does not match the reference namespace"
                    .to_string(),
            });
        }
        Ok(())
    }
}

impl Store for CobRepository {}

impl change::Storage for CobRepository {
    type StoreError = <git2::Repository as change::Storage>::StoreError;
    type LoadError = <git2::Repository as change::Storage>::LoadError;

    type ObjectId = <git2::Repository as change::Storage>::ObjectId;
    type Parent = <git2::Repository as change::Storage>::Parent;
    type Signatures = <git2::Repository as change::Storage>::Signatures;

    fn store<Signer>(
        &self,
        resource: Option<Self::Parent>,
        related: Vec<Self::Parent>,
        signer: &Signer,
        template: change::Template<Self::ObjectId>,
    ) -> Result<change::Entry, Self::StoreError>
    where
        Signer: radicle_crypto::signature::Signer<Self::Signatures>,
    {
        self.raw.store(resource, related, signer, template)
    }

    fn load(&self, id: Self::ObjectId) -> Result<change::Entry, Self::LoadError> {
        self.raw.load(id)
    }

    fn parents_of(&self, id: &Oid) -> Result<Vec<Oid>, Self::LoadError> {
        self.raw.parents_of(id)
    }

    fn manifest_of(&self, id: &Oid) -> Result<radicle_cob::Manifest, Self::LoadError> {
        self.raw.manifest_of(id)
    }
}

impl object::Storage for CobRepository {
    type ObjectsError = CobStoreError;
    type TypesError = CobStoreError;
    type UpdateError = CobStoreError;
    type RemoveError = CobStoreError;

    type Namespace = PublicKey;

    fn objects(
        &self,
        type_name: &TypeName,
        object_id: &ObjectId,
    ) -> Result<object::Objects, Self::ObjectsError> {
        let glob = format!("{COB_REF_PREFIX}/*/{COB_REF_MIDDLE}/{type_name}/{object_id}");
        let references = self
            .raw
            .references_glob(&glob)?
            .map(|reference| {
                let reference = reference?;
                self.validate_reference_change(&reference, type_name)?;
                Ok(Reference::try_from(reference)?)
            })
            .collect::<Result<Vec<_>, CobStoreError>>()?;
        Ok(references.into())
    }

    fn types(
        &self,
        type_name: &TypeName,
    ) -> Result<BTreeMap<ObjectId, object::Objects>, Self::TypesError> {
        let glob = format!("{COB_REF_PREFIX}/*/{COB_REF_MIDDLE}/{type_name}/*");
        let mut objects = BTreeMap::new();
        for reference in self.raw.references_glob(&glob)? {
            let reference = reference?;
            let name = reference.name().ok_or(CobStoreError::NonUtf8Reference)?;
            let formatted = RefString::try_from(name.to_string())?;
            let Some((observed_type, object_id)) = radicle_cob::object::parse_refstr(&formatted)
            else {
                return Err(CobStoreError::InvalidCobReference {
                    reference: name.to_string(),
                });
            };
            if observed_type != *type_name || RefStr::try_from_str(name).is_err() {
                return Err(CobStoreError::InvalidCobReference {
                    reference: name.to_string(),
                });
            }
            self.validate_reference_change(&reference, type_name)?;
            let converted = Reference::try_from(reference)?;
            objects
                .entry(object_id)
                .and_modify(|entries: &mut object::Objects| entries.push(converted.clone()))
                .or_insert_with(|| object::Objects::new(converted));
        }
        Ok(objects)
    }

    fn update(
        &self,
        namespace: &Self::Namespace,
        type_name: &TypeName,
        object_id: &ObjectId,
        entry: &change::EntryId,
    ) -> Result<(), Self::UpdateError> {
        let name = cob_ref_name(namespace, type_name, object_id)?;
        let new_target: git2::Oid = (*entry).into();
        match self.raw.find_reference(&name) {
            Ok(reference) => {
                let current =
                    reference
                        .target()
                        .ok_or_else(|| CobStoreError::InvalidCobReference {
                            reference: name.clone(),
                        })?;
                let operation = self.raw.find_commit(new_target)?;
                if !operation.parent_ids().any(|parent| parent == current) {
                    return Err(CobStoreError::ReferenceDrift { reference: name });
                }
                self.raw
                    .reference_matching(&name, new_target, true, current, REF_LOG_MESSAGE)?;
            }
            Err(error) if error.code() == git2::ErrorCode::NotFound => {
                self.raw
                    .reference(&name, new_target, false, REF_LOG_MESSAGE)?;
            }
            Err(error) => return Err(error.into()),
        }
        Ok(())
    }

    fn remove(
        &self,
        namespace: &Self::Namespace,
        type_name: &TypeName,
        object_id: &ObjectId,
    ) -> Result<(), Self::RemoveError> {
        let name = cob_ref_name(namespace, type_name, object_id)?;
        self.raw.find_reference(&name)?.delete()?;
        Ok(())
    }
}

fn namespace_from_ref(reference: &str) -> Result<PublicKey, CobStoreError> {
    let namespace = reference
        .split('/')
        .nth(NAMESPACE_COMPONENT_INDEX)
        .ok_or_else(|| CobStoreError::InvalidCobReference {
            reference: reference.to_string(),
        })?;
    PublicKey::from_str(namespace).map_err(|_| CobStoreError::InvalidCobReference {
        reference: reference.to_string(),
    })
}

fn cob_ref_name(
    namespace: &PublicKey,
    type_name: &TypeName,
    object_id: &ObjectId,
) -> Result<String, CobStoreError> {
    let candidate =
        format!("{COB_REF_PREFIX}/{namespace}/{COB_REF_MIDDLE}/{type_name}/{object_id}");
    Ok(RefString::try_from(candidate)?.to_string())
}
