use super::{step, step_assert, Pipeline};

pub fn pipeline() -> Pipeline {
    Pipeline {
        name: "hierarchy",
        summary: "Demonstrate component hierarchy and issue parent/child relationships",
        steps: vec![
            step_assert(
                "Create root component: Platform",
                &["--user", "admin@demo.com", "component", "create", "Platform", "--description", "Top-level platform component"],
                &["Platform"],
            ),
            step(
                "Grant admin on component 1",
                &["--user", "admin@demo.com", "acl", "set-component", "1",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permissions", "ADMIN_COMPONENTS"],
            ),
            step_assert(
                "Create child component: Frontend (under Platform)",
                &["--user", "admin@demo.com", "component", "create", "Frontend", "--description", "Frontend services", "--parent-id", "1"],
                &["Frontend"],
            ),
            step(
                "Grant admin on component 2",
                &["--user", "admin@demo.com", "acl", "set-component", "2",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permissions", "ADMIN_COMPONENTS"],
            ),
            step_assert(
                "Create child component: Backend (under Platform)",
                &["--user", "admin@demo.com", "component", "create", "Backend", "--description", "Backend services", "--parent-id", "1"],
                &["Backend"],
            ),
            step(
                "Grant admin on component 3",
                &["--user", "admin@demo.com", "acl", "set-component", "3",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permissions", "ADMIN_COMPONENTS"],
            ),
            step_assert(
                "Create child component: Mobile (under Platform)",
                &["--user", "admin@demo.com", "component", "create", "Mobile", "--description", "Mobile apps", "--parent-id", "1"],
                &["Mobile"],
            ),
            step(
                "Grant admin on component 4",
                &["--user", "admin@demo.com", "acl", "set-component", "4",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permissions", "ADMIN_COMPONENTS"],
            ),
            step(
                "List root components (should show Platform with 3 children)",
                &["--user", "admin@demo.com", "component", "list"],
            ),
            step(
                "List children of Platform",
                &["--user", "admin@demo.com", "component", "list", "--parent-id", "1"],
            ),
            step_assert(
                "Create an epic in Platform: 'Improve performance across all clients'",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Improve performance across all clients",
                    "--priority", "P1",
                    "--type", "EPIC",
                ],
                &["Improve performance across all clients"],
            ),
            step_assert(
                "Create a task in Frontend: 'Optimize bundle size'",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "2",
                    "--title", "Optimize bundle size",
                    "--priority", "P2",
                    "--type", "TASK",
                    "--assignee", "frontend-dev@example.com",
                ],
                &["Optimize bundle size"],
            ),
            step_assert(
                "Create a task in Backend: 'Add query caching layer'",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "3",
                    "--title", "Add query caching layer",
                    "--priority", "P1",
                    "--type", "TASK",
                    "--assignee", "backend-dev@example.com",
                ],
                &["Add query caching layer"],
            ),
            step_assert(
                "Create a task in Mobile: 'Reduce app startup time'",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "4",
                    "--title", "Reduce app startup time",
                    "--priority", "P2",
                    "--type", "TASK",
                    "--assignee", "mobile-dev@example.com",
                ],
                &["Reduce app startup time"],
            ),
            step(
                "Link tasks as children of the epic (task 2 -> epic 1)",
                &["--user", "admin@demo.com", "issue", "add-parent", "2", "1"],
            ),
            step(
                "Link task 3 as child of epic 1",
                &["--user", "admin@demo.com", "issue", "add-parent", "3", "1"],
            ),
            step(
                "Link task 4 as child of epic 1",
                &["--user", "admin@demo.com", "issue", "add-parent", "4", "1"],
            ),
            step(
                "Backend caching blocks frontend optimization (3 blocks 2)",
                &["--user", "admin@demo.com", "issue", "block", "3", "2"],
            ),
            step_assert(
                "List children of the epic",
                &["--user", "admin@demo.com", "issue", "children", "1"],
                &["Optimize bundle size"],
            ),
            step_assert(
                "List parents of task 2 (should show epic)",
                &["--user", "admin@demo.com", "issue", "parents", "2"],
                &["Improve performance"],
            ),
            step_assert(
                "View Platform component (should show child_count=3)",
                &["--user", "admin@demo.com", "component", "get", "1"],
                &["Platform"],
            ),
        ],
    }
}
