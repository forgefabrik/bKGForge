//! Composition root for bkgForge services.

use bkgforge_core::{DiscoveryId, DomainError, LaneId, ResourceRequirements, WorkflowId};
use bkgforge_discovery::{DiscoveryJob, DiscoveryState};
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

    /// Registers a discovery job and advances it to validation after security gating.
    ///
    /// This is a domain integration point only: no URL, HTTP, or browser action is
    /// performed until later discovery milestones provide an execution path.
    ///
    /// # Errors
    ///
    /// Returns an error when security pauses, blocks, or requires review, when
    /// the supplied job is not newly created, or when its identifier already exists.
    pub fn start_discovery(
        &mut self,
        mut job: DiscoveryJob,
        event: &SecurityEvent,
    ) -> Result<&DiscoveryJob, DomainError> {
        let decision = decide(event);
        match decision {
            SecurityDecision::Block | SecurityDecision::Pause | SecurityDecision::HumanReview => {
                return Err(DomainError::SecurityBlocked(format!(
                    "security decision: {decision:?}"
                )));
            }
            SecurityDecision::Allow | SecurityDecision::Monitor | SecurityDecision::Challenge => {}
        }
        if self
            .discovery_jobs
            .iter()
            .any(|current| current.id == job.id)
        {
            return Err(DomainError::Validation(format!(
                "discovery job already exists: {}",
                job.id.0
            )));
        }
        job.transition(DiscoveryState::Validating)?;
        self.discovery_jobs.push(job);
        self.discovery_jobs
            .last()
            .ok_or_else(|| DomainError::Validation("discovery job insertion failed".into()))
    }

    #[must_use]
    pub fn discovery_job(&self, id: &DiscoveryId) -> Option<&DiscoveryJob> {
        self.discovery_jobs.iter().find(|job| &job.id == id)
    }
    #[must_use]
    pub fn status(&self) -> RuntimeStatus {
        RuntimeStatus {
            ready_resources: self.resources.ready_count(),
            active_lanes: self.lanes.len(),
            discovery_jobs: self.discovery_jobs.len(),
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuntimeStatus {
    pub ready_resources: usize,
    pub active_lanes: usize,
    pub discovery_jobs: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bkgforge_core::{ExecutionMode, ExecutionPath, Resource, ResourceId, ResourceState};
    use bkgforge_discovery::DiscoveryRequest;
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

    #[test]
    fn runtime_registers_a_security_approved_discovery_job() {
        let mut runtime = Runtime::new(ResourcePool::default());
        let job = DiscoveryJob::new(
            DiscoveryId("discovery-1".into()),
            DiscoveryRequest {
                target: "https://example.invalid".into(),
                execution_mode: ExecutionMode::Simulation,
                preferred_path: ExecutionPath::Simulation,
                workflow: WorkflowId("discovery-v1".into()),
                resource_requirements: ResourceRequirements {
                    kind: "browser".into(),
                    required_labels: BTreeMap::new(),
                },
            },
        );
        let event = SecurityEvent {
            event_type: "discovery_requested".into(),
            identity_fingerprint: None,
            workflow_fingerprint: "discovery-v1".into(),
            repeated_failures: 0,
            velocity_per_minute: 0,
        };
        let created = runtime.start_discovery(job, &event).unwrap();
        assert_eq!(created.state, DiscoveryState::Validating);
        assert_eq!(runtime.status().discovery_jobs, 1);
    }
}
