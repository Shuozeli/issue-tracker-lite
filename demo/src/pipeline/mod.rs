pub mod access_control;
pub mod full_lifecycle;
pub mod groups;
pub mod hierarchy;
pub mod hotlists;
pub mod quickstart;
pub mod search;

/// A single step in a demo pipeline.
pub struct Step {
    /// Human-readable description of what this step demonstrates.
    pub description: &'static str,
    /// The CLI command arguments (everything after `it`).
    pub args: Vec<String>,
    /// If true, the step is expected to fail (demonstrating error handling).
    pub expect_failure: bool,
    /// Substrings that must appear in stdout for the step to be considered successful.
    pub assert_stdout_contains: Vec<String>,
}

/// A named demo pipeline.
pub struct Pipeline {
    pub name: &'static str,
    pub summary: &'static str,
    pub steps: Vec<Step>,
}

/// Helper to build a step concisely.
pub fn step(description: &'static str, args: &[&str]) -> Step {
    Step {
        description,
        args: args.iter().map(|s| s.to_string()).collect(),
        expect_failure: false,
        assert_stdout_contains: vec![],
    }
}

/// Helper to build a step with output verification.
pub fn step_assert(description: &'static str, args: &[&str], assert_contains: &[&str]) -> Step {
    Step {
        description,
        args: args.iter().map(|s| s.to_string()).collect(),
        expect_failure: false,
        assert_stdout_contains: assert_contains.iter().map(|s| s.to_string()).collect(),
    }
}

/// Helper to build a step that is expected to fail.
pub fn step_fail(description: &'static str, args: &[&str]) -> Step {
    Step {
        description,
        args: args.iter().map(|s| s.to_string()).collect(),
        expect_failure: true,
        assert_stdout_contains: vec![],
    }
}

/// Registry of all available pipelines.
pub fn all_pipelines() -> Vec<Pipeline> {
    vec![
        quickstart::pipeline(),
        hierarchy::pipeline(),
        hotlists::pipeline(),
        access_control::pipeline(),
        search::pipeline(),
        full_lifecycle::pipeline(),
        groups::pipeline(),
    ]
}
