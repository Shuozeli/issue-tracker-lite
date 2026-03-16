use super::{step, step_assert, Pipeline};

pub fn pipeline() -> Pipeline {
    Pipeline {
        name: "access-control",
        summary: "Set up ACLs and demonstrate permission checks",
        steps: vec![
            step_assert(
                "Create a component: Security",
                &["--user", "admin@demo.com", "component", "create", "Security", "--description", "Security-sensitive component"],
                &["Security"],
            ),
            step(
                "Grant admin on component 1",
                &["--user", "admin@demo.com", "acl", "set-component", "1",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permissions", "ADMIN_COMPONENTS"],
            ),
            step_assert(
                "Grant alice EDIT_ISSUES and VIEW_ISSUES on Security",
                &[
                    "--user", "admin@demo.com",
                    "acl", "set-component", "1",
                    "--identity-type", "user",
                    "--identity-value", "alice@example.com",
                    "--permissions", "VIEW_ISSUES,EDIT_ISSUES",
                ],
                &["VIEW_ISSUES", "EDIT_ISSUES"],
            ),
            step_assert(
                "Grant bob VIEW_ISSUES only on Security",
                &[
                    "--user", "admin@demo.com",
                    "acl", "set-component", "1",
                    "--identity-type", "user",
                    "--identity-value", "bob@example.com",
                    "--permissions", "VIEW_ISSUES",
                ],
                &["VIEW_ISSUES"],
            ),
            step_assert(
                "Grant public VIEW_COMPONENTS on Security",
                &[
                    "--user", "admin@demo.com",
                    "acl", "set-component", "1",
                    "--identity-type", "public",
                    "--identity-value", "*",
                    "--permissions", "VIEW_COMPONENTS",
                ],
                &["VIEW_COMPONENTS"],
            ),
            step(
                "View all ACL entries for Security",
                &["--user", "admin@demo.com", "acl", "get-component", "1"],
            ),
            step_assert(
                "Check alice's effective permissions (should have VIEW+COMMENT+EDIT due to implication)",
                &["--user", "admin@demo.com", "acl", "check", "1", "--user", "alice@example.com"],
                &["ACL"],
            ),
            step_assert(
                "Check bob's effective permissions (should have VIEW only)",
                &["--user", "admin@demo.com", "acl", "check", "1", "--user", "bob@example.com"],
                &["ACL"],
            ),
            step_assert(
                "Check carol's permissions (gets VIEW_COMPONENTS via PUBLIC ACL)",
                &["--user", "admin@demo.com", "acl", "check", "1", "--user", "carol@example.com"],
                &["VIEW_COMPONENTS"],
            ),
            step_assert(
                "Upgrade bob to ADMIN_ISSUES (upsert existing ACL entry)",
                &[
                    "--user", "admin@demo.com",
                    "acl", "set-component", "1",
                    "--identity-type", "user",
                    "--identity-value", "bob@example.com",
                    "--permissions", "ADMIN_ISSUES",
                ],
                &["ADMIN_ISSUES"],
            ),
            step_assert(
                "Check bob's new permissions (ADMIN implies EDIT, COMMENT, VIEW)",
                &["--user", "admin@demo.com", "acl", "check", "1", "--user", "bob@example.com"],
                &["ACL"],
            ),
            step_assert(
                "Create a hotlist: Security Audit",
                &[
                    "--user", "admin@demo.com",
                    "hotlist", "create",
                    "--name", "Security Audit",
                    "--description", "Critical security items requiring audit",
                    "--owner", "alice@example.com",
                ],
                &["Security Audit"],
            ),
            step(
                "Grant admin on hotlist 1",
                &["--user", "admin@demo.com", "acl", "set-hotlist", "1",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permission", "HOTLIST_ADMIN"],
            ),
            step(
                "Grant alice HOTLIST_ADMIN on Security Audit",
                &[
                    "--user", "admin@demo.com",
                    "acl", "set-hotlist", "1",
                    "--identity-type", "user",
                    "--identity-value", "alice@example.com",
                    "--permission", "HOTLIST_ADMIN",
                ],
            ),
            step(
                "Grant public HOTLIST_VIEW on Security Audit",
                &[
                    "--user", "admin@demo.com",
                    "acl", "set-hotlist", "1",
                    "--identity-type", "public",
                    "--identity-value", "*",
                    "--permission", "HOTLIST_VIEW",
                ],
            ),
            step(
                "View hotlist ACL entries",
                &["--user", "admin@demo.com", "acl", "get-hotlist", "1"],
            ),
            step(
                "Remove bob's component ACL entry",
                &[
                    "--user", "admin@demo.com",
                    "acl", "remove-component", "1",
                    "--identity-type", "user",
                    "--identity-value", "bob@example.com",
                ],
            ),
            step_assert(
                "Check bob after ACL removal (only PUBLIC VIEW_COMPONENTS remains)",
                &["--user", "admin@demo.com", "acl", "check", "1", "--user", "bob@example.com"],
                &["VIEW_COMPONENTS"],
            ),
        ],
    }
}
