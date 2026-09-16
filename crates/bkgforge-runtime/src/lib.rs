//! Composition root for bkgForge services.

use bkgforge_core::{DomainError, LaneId, ResourceRequirements, WorkflowId};
use bkgforge_discovery::DiscoveryJob;
use bkgforge_lanes::Lane;
use bkgforge_resources::ResourcePool;
use bkgforge_security::{SecurityDecision, SecurityEvent, decide};

pub struct Runtime {
    pub resources: ResourcePool,
    pub lanes: Vec<Lane>,
    pub discovery_jobs: Vec<DiscoveryJob>,
}
impl Runtime {
    #[must_use]
    pub fn new(resources: ResourcePool) -> Self {
        Self {
            resources,
            lanes: Vec::new(),
            discovery_jobs: Vec::new(),
        }
    }
    /// Security-gated admission for M1 discovery jobs; execution is owned by later adapters.
    ///
    /// # Errors
    ///
    /// Returns an error when security rejects the request or its lifecycle cannot advance.
    pub fn register_discovery(
        &mut self,
        mut job: DiscoveryJob,
        event: &SecurityEvent,
    ) -> Result<&DiscoveryJob, DomainError> {
        match decide(event) {
            SecurityDecision::Block | SecurityDecision::Pause | SecurityDecision::HumanReview => {
                return Err(DomainError::SecurityBlocked(
                    "discovery security gate rejected request".into(),
                ));
            }
            _ => {}
        }
        job.transition(bkgforge_core::DiscoveryState::Validating)?;
        self.discovery_jobs.push(job);
        self.discovery_jobs
            .last()
            .ok_or_else(|| DomainError::Validation("discovery insertion failed".into()))
    }
    /// Applies security policy then starts a lane with a resource binding.
    ///
    /// # Errors
    ///
    /// Returns an error when security pauses, blocks, or requires review, or
    /// when no required resource is available.
    pub fn start_lane(
        &mut self,
        id: LaneId,
        workflow: WorkflowId,
        requirements: &ResourceRequirements,
        event: &SecurityEvent,
    ) -> Result<&Lane, DomainError> {
        let decision = decide(event);
        match decision {
            SecurityDecision::Block | SecurityDecision::Pause | SecurityDecision::HumanReview => {
                return Err(DomainError::SecurityBlocked(format!(
                    "security decision: {decision:?}"
                )));
            }
            SecurityDecision::Allow | SecurityDecision::Monitor | SecurityDecision::Challenge => {}
        }
        let mut lane = Lane::new(id, workflow);
        lane.bind(&mut self.resources, requirements)?;
        self.lanes.push(lane);
        self.lanes
            .last()
            .ok_or_else(|| DomainError::Validation("lane insertion failed".into()))
    }
    #[must_use]
    pub fn status(&self) -> RuntimeStatus {
        RuntimeStatus {
            ready_resources: self.resources.ready_count(),
            active_lanes: self.lanes.len(),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeStatus {
    pub ready_resources: usize,
    pub active_lanes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkgforge_core::{Resource, ResourceId, ResourceState};
    use std::collections::BTreeMap;
    #[test]
    fn security_block_prevents_lane_start() {
        let pool = ResourcePool::new([Resource {
            id: ResourceId("r".into()),
            kind: "browser".into(),
            state: ResourceState::Ready,
            capacity: 1,
            labels: BTreeMap::new(),
        }]);
        let mut runtime = Runtime::new(pool);
        let result = runtime.start_lane(
            LaneId("l".into()),
            WorkflowId("w".into()),
            &ResourceRequirements {
                kind: "browser".into(),
                required_labels: BTreeMap::new(),
            },
            &SecurityEvent {
                event_type: "test".into(),
                identity_fingerprint: None,
                workflow_fingerprint: "w".into(),
                repeated_failures: 5,
                velocity_per_minute: 0,
            },
        );
        assert!(result.is_err());
    }
}
