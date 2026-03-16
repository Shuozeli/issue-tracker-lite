use super::{step, step_assert, Pipeline};

pub fn pipeline() -> Pipeline {
    Pipeline {
        name: "hotlists",
        summary: "Create hotlists, add issues, reorder priorities",
        steps: vec![
            step_assert(
                "Create a component for our issues",
                &["--user", "admin@demo.com", "component", "create", "WebApp", "--description", "Web application"],
                &["WebApp"],
            ),
            step(
                "Grant admin on component 1",
                &["--user", "admin@demo.com", "acl", "set-component", "1",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permissions", "ADMIN_COMPONENTS"],
            ),
            step_assert(
                "Create issue: Fix CSRF vulnerability",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Fix CSRF vulnerability in form submissions",
                    "--priority", "P0",
                    "--type", "VULNERABILITY",
                ],
                &["Fix CSRF vulnerability in form submissions"],
            ),
            step_assert(
                "Create issue: Upgrade database driver",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Upgrade database driver to v5",
                    "--priority", "P2",
                    "--type", "INTERNAL_CLEANUP",
                ],
                &["Upgrade database driver to v5"],
            ),
            step_assert(
                "Create issue: Add rate limiting",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Add rate limiting to public API endpoints",
                    "--priority", "P1",
                    "--type", "FEATURE_REQUEST",
                ],
                &["Add rate limiting to public API endpoints"],
            ),
            step_assert(
                "Create issue: Migrate to new auth provider",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Migrate to new auth provider",
                    "--priority", "P1",
                    "--type", "TASK",
                ],
                &["Migrate to new auth provider"],
            ),
            step_assert(
                "Create hotlist: 'Sprint 12' for current sprint work",
                &[
                    "--user", "admin@demo.com",
                    "hotlist", "create",
                    "--name", "Sprint 12",
                    "--description", "Sprint 12 deliverables",
                    "--owner", "pm@example.com",
                ],
                &["Sprint 12"],
            ),
            step(
                "Grant admin on hotlist 1",
                &["--user", "admin@demo.com", "acl", "set-hotlist", "1",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permission", "HOTLIST_ADMIN"],
            ),
            step_assert(
                "Add CSRF fix to Sprint 12",
                &["--user", "admin@demo.com", "hotlist", "add-issue", "1", "1", "--by", "pm@example.com"],
                &["added to hotlist"],
            ),
            step_assert(
                "Add rate limiting to Sprint 12",
                &["--user", "admin@demo.com", "hotlist", "add-issue", "1", "3", "--by", "pm@example.com"],
                &["added to hotlist"],
            ),
            step_assert(
                "Add auth migration to Sprint 12",
                &["--user", "admin@demo.com", "hotlist", "add-issue", "1", "4", "--by", "pm@example.com"],
                &["added to hotlist"],
            ),
            step_assert(
                "List issues in Sprint 12 (default order)",
                &["--user", "admin@demo.com", "hotlist", "issues", "1"],
                &["pm@example.com"],
            ),
            step(
                "Reorder Sprint 12: CSRF first, then auth, then rate limiting",
                &["--user", "admin@demo.com", "hotlist", "reorder", "1", "--order", "1,4,3"],
            ),
            step_assert(
                "List Sprint 12 again (new order)",
                &["--user", "admin@demo.com", "hotlist", "issues", "1"],
                &["pm@example.com"],
            ),
            step_assert(
                "Create a Tech Debt hotlist",
                &[
                    "--user", "admin@demo.com",
                    "hotlist", "create",
                    "--name", "Tech Debt",
                    "--description", "Technical debt to address when capacity allows",
                    "--owner", "tech-lead@example.com",
                ],
                &["Tech Debt"],
            ),
            step(
                "Grant admin on hotlist 2",
                &["--user", "admin@demo.com", "acl", "set-hotlist", "2",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permission", "HOTLIST_ADMIN"],
            ),
            step_assert(
                "Add DB driver upgrade to Tech Debt",
                &["--user", "admin@demo.com", "hotlist", "add-issue", "2", "2", "--by", "tech-lead@example.com"],
                &["added to hotlist"],
            ),
            step(
                "List all hotlists",
                &["--user", "admin@demo.com", "hotlist", "list"],
            ),
            step_assert(
                "View Sprint 12 details",
                &["--user", "admin@demo.com", "hotlist", "get", "1"],
                &["Sprint 12"],
            ),
            step(
                "Remove rate limiting from Sprint 12 (descoped)",
                &["--user", "admin@demo.com", "hotlist", "remove-issue", "1", "3"],
            ),
            step_assert(
                "Sprint 12 after removing descoped issue",
                &["--user", "admin@demo.com", "hotlist", "issues", "1"],
                &["pm@example.com"],
            ),
        ],
    }
}
