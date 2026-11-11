//! Exact byte-component relative paths.

/// Pure path-admission failures.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PathError {
    /// No components were supplied.
    EmptyPath,
    /// A component was empty.
    EmptyComponent,
    /// A component was `.`.
    CurrentDirectory,
    /// A component was `..`.
    ParentDirectory,
    /// A component contained NUL.
    Nul,
    /// A component contained `/`.
    ForwardSlash,
    /// A component contained `\`.
    Backslash,
    /// Path byte accounting overflowed.
    ByteCountOverflow,
}

/// An admitted non-empty relative path with exact byte components.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RelativePath {
    components: alloc::vec::Vec<alloc::vec::Vec<u8>>,
    encoded_bytes: u64,
}

impl RelativePath {
    /// Admits exact non-empty path components.
    ///
    /// # Errors
    ///
    /// Returns a typed path error for malformed components or byte overflow.
    pub fn new(components: alloc::vec::Vec<alloc::vec::Vec<u8>>) -> Result<Self, PathError> {
        if components.is_empty() {
            return Err(PathError::EmptyPath);
        }
        let encoded_bytes = encoded_byte_count(&components)?;
        Ok(Self {
            components,
            encoded_bytes,
        })
    }

    /// Returns exact path components.
    #[must_use]
    pub fn components(&self) -> &[alloc::vec::Vec<u8>] {
        &self.components
    }

    /// Returns encoded bytes, including separators between components.
    #[must_use]
    pub const fn encoded_bytes(&self) -> u64 {
        self.encoded_bytes
    }

    /// Returns the path depth.
    ///
    /// # Errors
    ///
    /// Returns an accounting error when the component count does not fit `u64`.
    pub fn depth(&self) -> Result<u64, PathError> {
        u64::try_from(self.components.len()).map_err(|_| PathError::ByteCountOverflow)
    }

    /// Returns the parent path, if this path is not at the root level.
    ///
    /// # Errors
    ///
    /// Returns a path error when invariant-preserving parent reconstruction fails.
    pub fn parent(&self) -> Result<Option<Self>, PathError> {
        if self.components.len() == 1 {
            return Ok(None);
        }
        let parent_count = self.components.len().saturating_sub(1);
        Self::new(self.components[..parent_count].to_vec()).map(Some)
    }
}

fn encoded_byte_count(components: &[alloc::vec::Vec<u8>]) -> Result<u64, PathError> {
    let mut encoded_bytes = 0_u64;
    for (index, component) in components.iter().enumerate() {
        validate_component(component)?;
        let component_bytes =
            u64::try_from(component.len()).map_err(|_| PathError::ByteCountOverflow)?;
        encoded_bytes = encoded_bytes
            .checked_add(component_bytes)
            .ok_or(PathError::ByteCountOverflow)?;
        if index > 0 {
            encoded_bytes = encoded_bytes
                .checked_add(1)
                .ok_or(PathError::ByteCountOverflow)?;
        }
    }
    Ok(encoded_bytes)
}

fn validate_component(component: &[u8]) -> Result<(), PathError> {
    if component.is_empty() {
        return Err(PathError::EmptyComponent);
    }
    validate_reserved_component(component)?;
    validate_component_bytes(component)
}

pub(crate) fn validate_reserved_component(component: &[u8]) -> Result<(), PathError> {
    if component == b"." {
        return Err(PathError::CurrentDirectory);
    }
    if component == b".." {
        return Err(PathError::ParentDirectory);
    }
    Ok(())
}

fn validate_component_bytes(component: &[u8]) -> Result<(), PathError> {
    if component.contains(&0) {
        return Err(PathError::Nul);
    }
    if component.contains(&b'/') {
        return Err(PathError::ForwardSlash);
    }
    if component.contains(&b'\\') {
        return Err(PathError::Backslash);
    }
    Ok(())
}
