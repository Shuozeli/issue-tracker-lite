use super::{step, step_assert, Pipeline};

pub fn pipeline() -> Pipeline {
    Pipeline {
        name: "quickstart",
        summary: "Create a component, file issues, update status, and add comments",
        steps: vec![
            step(
                "Check server health",
                &["--user", "admin@demo.com", "ping"],
            ),
            step_assert(
                "Create a component for our project",
                &["--user", "admin@demo.com", "component", "create", "MyProject", "--description", "Main project component"],
                &["MyProject"],
            ),
            step(
                "Grant admin permissions on the component",
                &["--user", "admin@demo.com", "acl", "set-component", "1",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permissions", "ADMIN_COMPONENTS"],
            ),
            step_assert(
                "File a P0 bug: login page crashes on empty password",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Login page crashes on empty password",
                    "--description", "Submitting the login form with an empty password field causes a 500 error",
                    "--priority", "P0",
                    "--type", "BUG",
                    "--reporter", "qa@example.com",
                ],
                &["Login page crashes on empty password"],
            ),
            step_assert(
                "File a P2 feature request: dark mode support",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Add dark mode support",
                    "--description", "Users have requested a dark mode theme option",
                    "--priority", "P2",
                    "--type", "FEATURE_REQUEST",
                    "--reporter", "pm@example.com",
                ],
                &["Add dark mode support"],
            ),
            step_assert(
                "File a P1 bug: memory leak in background sync",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Memory leak in background sync worker",
                    "--description", "RSS grows ~50MB/hour when sync is running",
                    "--priority", "P1",
                    "--type", "BUG",
                    "--severity", "S1",
                    "--reporter", "ops@example.com",
                ],
                &["Memory leak in background sync worker"],
            ),
            step(
                "List all open issues in MyProject",
                &["--user", "admin@demo.com", "issue", "list", "--component", "1"],
            ),
            step_assert(
                "Assign the P0 bug to alice and start work",
                &[
                    "--user", "admin@demo.com",
                    "issue", "update", "1",
                    "--assignee", "alice@example.com",
                    "--status", "IN_PROGRESS",
                ],
                &["IN_PROGRESS"],
            ),
            step_assert(
                "Alice adds an investigation comment",
                &[
                    "--user", "admin@demo.com",
                    "comment", "add", "1",
                    "--body", "Reproduced on Chrome 120. Root cause: missing null check in password validator.",
                    "--author", "alice@example.com",
                ],
                &["missing null check"],
            ),
            step_assert(
                "Alice fixes the bug and marks it FIXED",
                &["--user", "admin@demo.com", "issue", "update", "1", "--status", "FIXED"],
                &["FIXED"],
            ),
            step_assert(
                "View the final state of the fixed issue",
                &["--user", "admin@demo.com", "issue", "get", "1"],
                &["FIXED"],
            ),
            step(
                "List all comments on the issue",
                &["--user", "admin@demo.com", "comment", "list", "1"],
            ),
            step_assert(
                "Search for issues mentioning 'memory'",
                &["--user", "admin@demo.com", "search", "memory"],
                &["Memory leak"],
            ),
        ],
    }
}
