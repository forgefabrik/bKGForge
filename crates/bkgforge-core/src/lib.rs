//! Provider-neutral domain contracts for bkgForge.

use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ResourceId(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LaneId(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct WorkflowId(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SecretRef(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DiscoveryId(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct SiteId(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct EndpointId(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TraceId(pub String);
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ArtifactId(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceState {
    Discovered,
    Ready,
    Active,
    Degraded,
    Exhausting,
    Cooldown,
    Invalid,
    Disabled,
    Cleanup,
    Deleted,
}
impl ResourceState {
    #[must_use]
    pub const fn allocatable(self) -> bool {
        matches!(self, Self::Ready)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaneState {
    WaitingForResource,
    Active,
    Degraded,
    Rebinding,
    Recovering,
    Paused,
    Failed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExecutionMode {
    Real,
    Simulation,
    Replay,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ExecutionPath {
    DirectApi,
    Sdk,
    Cli,
    Browser,
    Replay,
    Simulation,
}

/// Provider-neutral lifecycle for work that discovers a source into a model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiscoveryState {
    Created,
    Validating,
    Provisioning,
    Capturing,
    Analyzing,
    Modeling,
    Generating,
    Completed,
    Failed,
    Cancelled,
}
impl DiscoveryState {
    #[must_use]
    pub const fn terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    #[must_use]
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Validating | Self::Cancelled)
                | (
                    Self::Validating,
                    Self::Provisioning | Self::Failed | Self::Cancelled
                )
                | (
                    Self::Provisioning,
                    Self::Capturing | Self::Failed | Self::Cancelled
                )
                | (
                    Self::Capturing,
                    Self::Analyzing | Self::Failed | Self::Cancelled
                )
                | (
                    Self::Analyzing,
                    Self::Modeling | Self::Failed | Self::Cancelled
                )
                | (
                    Self::Modeling,
                    Self::Generating | Self::Failed | Self::Cancelled
                )
                | (
                    Self::Generating,
                    Self::Completed | Self::Failed | Self::Cancelled
                )
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resource {
    pub id: ResourceId,
    pub kind: String,
    pub state: ResourceState,
    pub capacity: u32,
    pub labels: BTreeMap<String, String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceRequirements {
    pub kind: String,
    pub required_labels: BTreeMap<String, String>,
}
impl ResourceRequirements {
    #[must_use]
    pub fn matches(&self, resource: &Resource) -> bool {
        resource.kind == self.kind
            && resource.state.allocatable()
            && resource.capacity > 0
            && self
                .required_labels
                .iter()
                .all(|(key, value)| resource.labels.get(key) == Some(value))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainError {
    ResourceUnavailable {
        kind: String,
    },
    InvalidTransition {
        from: LaneState,
        to: LaneState,
    },
    Validation(String),
    SecurityBlocked(String),
    InvalidDiscoveryTransition {
        from: DiscoveryState,
        to: DiscoveryState,
    },
}
impl Display for DomainError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ResourceUnavailable { kind } => {
                write!(f, "no ready resource available for {kind}")
            }
            Self::InvalidTransition { from, to } => {
                write!(f, "invalid lane transition: {from:?} -> {to:?}")
            }
            Self::Validation(message) | Self::SecurityBlocked(message) => f.write_str(message),
            Self::InvalidDiscoveryTransition { from, to } => {
                write!(f, "invalid discovery transition: {from:?} -> {to:?}")
            }
        }
    }
}
impl std::error::Error for DomainError {}

/// A provider adapter boundary. Implementations live outside the core.
pub trait Provider: Send + Sync {
    fn name(&self) -> &str;
    fn supported_paths(&self) -> Vec<ExecutionPath>;
}
/// An interchangeable store for secret values. Domain objects retain only `SecretRef`s.
pub trait SecretStore: Send + Sync {
    fn contains(&self, reference: &SecretRef) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requirement_only_matches_ready_capacity() {
        let r = Resource {
            id: ResourceId("r1".into()),
            kind: "browser".into(),
            state: ResourceState::Ready,
            capacity: 1,
            labels: BTreeMap::new(),
        };
        assert!(
            ResourceRequirements {
                kind: "browser".into(),
                required_labels: BTreeMap::new()
            }
            .matches(&r)
        );
    }
}
