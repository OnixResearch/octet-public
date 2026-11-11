use std::{collections::BTreeMap, path::Path};

use radicle_cob::{
    change::{self, Entry},
    object::{self, Reference},
    ObjectId, Store, TypeName,
};
use radicle_crypto::PublicKey;
use radicle_oid::Oid;
use thiserror::Error;

const COB_REF_PREFIX: &str = "refs/namespaces";
const COB_REF_MIDDLE: &str = "refs/cobs";
const NAMESPACE_COMPONENT_INDEX: usize = 2;

pub(crate) struct ReferenceRepository {
    raw: git2::Repository,
}

impl ReferenceRepository {
    pub(crate) fn open(path: &Path) -> Result<Self, git2::Error> {
        Ok(Self {
            raw: git2::Repository::open(path)?,
        })
    }
}

impl Store for ReferenceRepository {}

impl change::Storage for ReferenceRepository {
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
    ) -> Result<Entry, Self::StoreError>
    where
        Signer: radicle_crypto::signature::Signer<Self::Signatures>,
    {
        self.raw.store(resource, related, signer, template)
    }

    fn load(&self, id: Self::ObjectId) -> Result<Entry, Self::LoadError> {
        self.raw.load(id)
    }

    fn parents_of(&self, id: &Oid) -> Result<Vec<Oid>, Self::LoadError> {
        self.raw.parents_of(id)
    }

    fn manifest_of(&self, id: &Oid) -> Result<radicle_cob::Manifest, Self::LoadError> {
        self.raw.manifest_of(id)
    }
}

impl object::Storage for ReferenceRepository {
    type ObjectsError = ReferenceStoreError;
    type TypesError = ReferenceStoreError;
    type UpdateError = ReferenceStoreError;
    type RemoveError = ReferenceStoreError;
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
                validate_reference_namespace(&self.raw, &reference)?;
                Ok(Reference::try_from(reference)?)
            })
            .collect::<Result<Vec<_>, ReferenceStoreError>>()?;
        Ok(references.into())
    }

    fn types(
        &self,
        _type_name: &TypeName,
    ) -> Result<BTreeMap<ObjectId, object::Objects>, Self::TypesError> {
        Err(ReferenceStoreError::ReadOnlyOperation)
    }

    fn update(
        &self,
        _namespace: &Self::Namespace,
        _type_name: &TypeName,
        _object_id: &ObjectId,
        _entry: &change::EntryId,
    ) -> Result<(), Self::UpdateError> {
        Err(ReferenceStoreError::ReadOnlyOperation)
    }

    fn remove(
        &self,
        _namespace: &Self::Namespace,
        _type_name: &TypeName,
        _object_id: &ObjectId,
    ) -> Result<(), Self::RemoveError> {
        Err(ReferenceStoreError::ReadOnlyOperation)
    }
}

#[derive(Debug, Error)]
pub(crate) enum ReferenceStoreError {
    #[error(transparent)]
    Git(#[from] git2::Error),
    #[error(transparent)]
    Convert(#[from] object::storage::convert::Error),
    #[error("reference namespace does not match signed change author: {reference}")]
    NamespaceMismatch { reference: String },
    #[error("reference consumer is read-only")]
    ReadOnlyOperation,
    #[error("reference name is not UTF-8")]
    NonUtf8Reference,
    #[error("reference has no direct target: {reference}")]
    MissingTarget { reference: String },
    #[error("reference contains an invalid signed change: {reference}: {reason}")]
    InvalidChange { reference: String, reason: String },
}

fn validate_reference_namespace(
    repository: &git2::Repository,
    reference: &git2::Reference<'_>,
) -> Result<(), ReferenceStoreError> {
    let name = reference
        .name()
        .ok_or(ReferenceStoreError::NonUtf8Reference)?;
    let namespace = name
        .split('/')
        .nth(NAMESPACE_COMPONENT_INDEX)
        .ok_or(ReferenceStoreError::NonUtf8Reference)?;
    let target = reference
        .target()
        .ok_or_else(|| ReferenceStoreError::MissingTarget {
            reference: name.to_string(),
        })?;
    let entry = <git2::Repository as change::Storage>::load(repository, target.into()).map_err(
        |error| ReferenceStoreError::InvalidChange {
            reference: name.to_string(),
            reason: error.to_string(),
        },
    )?;
    if entry.author().to_string() == namespace {
        Ok(())
    } else {
        Err(ReferenceStoreError::NamespaceMismatch {
            reference: name.to_string(),
        })
    }
}
