//! Discovery domain lifecycle contracts.
//!
//! This crate intentionally models jobs only. HTTP, browser capture, traces,
//! schemas, and generated artifacts are introduced by their respective later
//! milestones and are not fabricated here.

pub use bkgforge_core::DiscoveryState;
use bkgforge_core::{
    ArtifactId, DiscoveryId, DomainError, ExecutionMode, ExecutionPath, ResourceRequirements,
    SiteId, TraceId, WorkflowId,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryRequest {
    /// The source identifier to be examined by a later discovery execution path.
    pub target: String,
    pub execution_mode: ExecutionMode,
    pub preferred_path: ExecutionPath,
    pub workflow: WorkflowId,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryJob {
    pub id: DiscoveryId,
    pub request: DiscoveryRequest,
    pub state: DiscoveryState,
    pub failure: Option<String>,
}

impl DiscoveryJob {
    #[must_use]
    pub fn new(id: DiscoveryId, request: DiscoveryRequest) -> Self {
        Self {
            id,
            request,
            state: DiscoveryState::Created,
            failure: None,
        }
    }

    /// Advances the discovery lifecycle without executing an integration.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidDiscoveryTransition`] for unsupported or
    /// terminal-state transitions.
    pub fn transition(&mut self, next: DiscoveryState) -> Result<(), DomainError> {
        if !self.state.can_transition_to(next) {
            return Err(DomainError::InvalidDiscoveryTransition {
                from: self.state,
                to: next,
            });
        }
        self.state = next;
        Ok(())
    }

    /// Marks a non-terminal job as failed and records an operational reason.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidDiscoveryTransition`] when the job is terminal.
    pub fn fail(&mut self, reason: impl Into<String>) -> Result<(), DomainError> {
        self.transition(DiscoveryState::Failed)?;
        self.failure = Some(reason.into());
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryResult {
    pub discovery_id: DiscoveryId,
    pub site_id: Option<SiteId>,
    pub trace_id: Option<TraceId>,
    pub artifact_ids: Vec<ArtifactId>,
    pub execution_mode: ExecutionMode,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn request() -> DiscoveryRequest {
        DiscoveryRequest {
            target: "https://example.invalid".into(),
            execution_mode: ExecutionMode::Simulation,
            preferred_path: ExecutionPath::Simulation,
            workflow: WorkflowId("discovery-v1".into()),
            resource_requirements: ResourceRequirements {
                kind: "browser".into(),
                required_labels: BTreeMap::new(),
            },
        }
    }

    #[test]
    fn job_reaches_completed_through_the_declared_lifecycle() {
        let mut job = DiscoveryJob::new(DiscoveryId("discovery-1".into()), request());
        for state in [
            DiscoveryState::Validating,
            DiscoveryState::Provisioning,
            DiscoveryState::Capturing,
            DiscoveryState::Analyzing,
            DiscoveryState::Modeling,
            DiscoveryState::Generating,
            DiscoveryState::Completed,
        ] {
            job.transition(state).unwrap();
        }
        assert_eq!(job.state, DiscoveryState::Completed);
        assert!(job.state.terminal());
    }

    #[test]
    fn failure_is_available_from_an_active_state() {
        let mut job = DiscoveryJob::new(DiscoveryId("discovery-1".into()), request());
        job.transition(DiscoveryState::Validating).unwrap();
        job.fail("target validation rejected the request").unwrap();
        assert_eq!(job.state, DiscoveryState::Failed);
        assert_eq!(
            job.failure.as_deref(),
            Some("target validation rejected the request")
        );
    }

    #[test]
    fn invalid_transition_is_rejected() {
        let mut job = DiscoveryJob::new(DiscoveryId("discovery-1".into()), request());
        assert!(job.transition(DiscoveryState::Completed).is_err());
    }
}
