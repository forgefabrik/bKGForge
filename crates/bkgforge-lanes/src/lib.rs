//! Long-lived workflow lanes that retain identity while resources are rebound.

use bkgforge_core::{DomainError, LaneId, LaneState, ResourceId, ResourceRequirements, WorkflowId};
use bkgforge_resources::ResourcePool;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Lane {
    pub id: LaneId,
    pub workflow: WorkflowId,
    pub state: LaneState,
    pub binding: Option<ResourceId>,
}
impl Lane {
    #[must_use]
    pub fn new(id: LaneId, workflow: WorkflowId) -> Self {
        Self {
            id,
            workflow,
            state: LaneState::WaitingForResource,
            binding: None,
        }
    }
    /// Acquires a resource without changing the lane identity.
    ///
    /// # Errors
    ///
    /// Returns a resource allocation error when no matching resource is ready.
    pub fn bind(
        &mut self,
        pool: &mut ResourcePool,
        requirements: &ResourceRequirements,
    ) -> Result<(), DomainError> {
        self.state = LaneState::Rebinding;
        let resource = pool.acquire(requirements)?;
        self.binding = Some(resource.id);
        self.state = LaneState::Active;
        Ok(())
    }
    /// Degrades the current binding and attempts to bind a replacement.
    ///
    /// # Errors
    ///
    /// Returns a resource allocation error and leaves the lane waiting when no
    /// replacement is available.
    pub fn recover(
        &mut self,
        pool: &mut ResourcePool,
        requirements: &ResourceRequirements,
    ) -> Result<(), DomainError> {
        self.state = LaneState::Degraded;
        if let Some(id) = self.binding.take() {
            pool.degrade(&id);
        }
        self.bind(pool, requirements)
            .inspect_err(|_| self.state = LaneState::WaitingForResource)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkgforge_core::{Resource, ResourceState};
    use std::collections::BTreeMap;
    #[test]
    fn lane_survives_rebinding() {
        let mut pool = ResourcePool::new([
            Resource {
                id: ResourceId("old".into()),
                kind: "browser".into(),
                state: ResourceState::Ready,
                capacity: 1,
                labels: BTreeMap::new(),
            },
            Resource {
                id: ResourceId("new".into()),
                kind: "browser".into(),
                state: ResourceState::Ready,
                capacity: 1,
                labels: BTreeMap::new(),
            },
        ]);
        let req = ResourceRequirements {
            kind: "browser".into(),
            required_labels: BTreeMap::new(),
        };
        let mut lane = Lane::new(LaneId("lane-1".into()), WorkflowId("w".into()));
        lane.bind(&mut pool, &req).unwrap();
        let first_binding = lane.binding.clone();
        lane.recover(&mut pool, &req).unwrap();
        assert_eq!(lane.id, LaneId("lane-1".into()));
        assert_ne!(lane.binding, first_binding);
    }
}
