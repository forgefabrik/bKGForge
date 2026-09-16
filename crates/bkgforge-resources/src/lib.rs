//! Generic resource pool allocation, release, and rebinding primitives.

use bkgforge_core::{DomainError, Resource, ResourceId, ResourceRequirements, ResourceState};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct ResourcePool {
    resources: BTreeMap<ResourceId, Resource>,
}
impl ResourcePool {
    #[must_use]
    pub fn new(resources: impl IntoIterator<Item = Resource>) -> Self {
        Self {
            resources: resources.into_iter().map(|r| (r.id.clone(), r)).collect(),
        }
    }
    /// Acquires a matching ready resource and consumes one unit of capacity.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ResourceUnavailable`] when no matching resource is ready.
    pub fn acquire(
        &mut self,
        requirements: &ResourceRequirements,
    ) -> Result<Resource, DomainError> {
        let candidate = self
            .resources
            .values_mut()
            .find(|resource| requirements.matches(resource))
            .ok_or_else(|| DomainError::ResourceUnavailable {
                kind: requirements.kind.clone(),
            })?;
        candidate.capacity -= 1;
        if candidate.capacity == 0 {
            candidate.state = ResourceState::Active;
        }
        Ok(candidate.clone())
    }
    /// Returns capacity to a resource and marks it ready.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ResourceUnavailable`] if the resource is unknown.
    pub fn release(&mut self, id: &ResourceId) -> Result<(), DomainError> {
        let resource =
            self.resources
                .get_mut(id)
                .ok_or_else(|| DomainError::ResourceUnavailable {
                    kind: "unknown".into(),
                })?;
        resource.capacity += 1;
        resource.state = ResourceState::Ready;
        Ok(())
    }
    pub fn degrade(&mut self, id: &ResourceId) {
        if let Some(resource) = self.resources.get_mut(id) {
            resource.state = ResourceState::Degraded;
        }
    }
    #[must_use]
    pub fn ready_count(&self) -> usize {
        self.resources
            .values()
            .filter(|r| r.state == ResourceState::Ready)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    #[test]
    fn release_makes_resource_reusable() {
        let id = ResourceId("a".into());
        let mut pool = ResourcePool::new([Resource {
            id: id.clone(),
            kind: "session".into(),
            state: ResourceState::Ready,
            capacity: 1,
            labels: BTreeMap::new(),
        }]);
        let req = ResourceRequirements {
            kind: "session".into(),
            required_labels: BTreeMap::new(),
        };
        pool.acquire(&req).unwrap();
        assert!(pool.acquire(&req).is_err());
        pool.release(&id).unwrap();
        assert!(pool.acquire(&req).is_ok());
    }
}
