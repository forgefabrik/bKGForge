//! Validated, provider-neutral workflow IR and execution-path selection.

use bkgforge_core::{DomainError, ExecutionMode, ExecutionPath, WorkflowId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowStepKind {
    Navigate,
    Click,
    Fill,
    Select,
    Wait,
    Extract,
    Http,
    Authenticate,
    Verify,
    CreateResource,
    Validate,
    Cleanup,
    HumanApproval,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SecurityClassification {
    Public,
    Sensitive,
    Critical,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub backoff_ms: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowStep {
    pub kind: WorkflowStepKind,
    pub input: String,
    pub output: Option<String>,
    pub timeout_ms: u64,
    pub retry: RetryPolicy,
    pub security: SecurityClassification,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Workflow {
    pub id: WorkflowId,
    pub version: u32,
    pub steps: Vec<WorkflowStep>,
    pub validated: bool,
    pub experimental: bool,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompiledWorkflow {
    pub workflow: Workflow,
}

/// Validates minimum executable workflow invariants.
///
/// # Errors
///
/// Returns [`DomainError::Validation`] for missing steps or invalid step policies.
pub fn validate(workflow: &Workflow) -> Result<(), DomainError> {
    if workflow.steps.is_empty() {
        return Err(DomainError::Validation(
            "workflow must contain at least one step".into(),
        ));
    }
    if workflow
        .steps
        .iter()
        .any(|step| step.timeout_ms == 0 || step.retry.max_attempts == 0)
    {
        return Err(DomainError::Validation(
            "each step needs a positive timeout and retry budget".into(),
        ));
    }
    Ok(())
}
/// Produces an executable workflow only from validated or explicit experimental IR.
///
/// # Errors
///
/// Returns [`DomainError::Validation`] when IR validation fails or experimental
/// intent has not been declared.
pub fn compile(mut workflow: Workflow) -> Result<CompiledWorkflow, DomainError> {
    validate(&workflow)?;
    if !workflow.validated && !workflow.experimental {
        return Err(DomainError::Validation(
            "unvalidated workflows must be explicitly experimental".into(),
        ));
    }
    workflow.validated = true;
    Ok(CompiledWorkflow { workflow })
}
#[must_use]
pub fn plan_path(mode: ExecutionMode, available: &[ExecutionPath]) -> Option<ExecutionPath> {
    match mode {
        ExecutionMode::Replay => available
            .contains(&ExecutionPath::Replay)
            .then_some(ExecutionPath::Replay),
        ExecutionMode::Simulation => available
            .contains(&ExecutionPath::Simulation)
            .then_some(ExecutionPath::Simulation),
        ExecutionMode::Real => [
            ExecutionPath::DirectApi,
            ExecutionPath::Sdk,
            ExecutionPath::Cli,
            ExecutionPath::Browser,
        ]
        .into_iter()
        .find(|path| available.contains(path)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn step() -> WorkflowStep {
        WorkflowStep {
            kind: WorkflowStepKind::Verify,
            input: "subject".into(),
            output: None,
            timeout_ms: 10,
            retry: RetryPolicy {
                max_attempts: 1,
                backoff_ms: 0,
            },
            security: SecurityClassification::Public,
        }
    }
    #[test]
    fn real_prefers_direct_api_over_browser() {
        assert_eq!(
            plan_path(
                ExecutionMode::Real,
                &[ExecutionPath::Browser, ExecutionPath::DirectApi]
            ),
            Some(ExecutionPath::DirectApi)
        );
    }
    #[test]
    fn compiler_rejects_unvalidated_non_experimental_workflow() {
        assert!(
            compile(Workflow {
                id: WorkflowId("x".into()),
                version: 1,
                steps: vec![step()],
                validated: false,
                experimental: false
            })
            .is_err()
        );
    }
}
