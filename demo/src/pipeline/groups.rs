use super::{step, step_assert, step_fail, Pipeline};

pub fn pipeline() -> Pipeline {
    Pipeline {
        name: "groups",
        summary: "Create groups, manage memberships, resolve transitive membership, and integrate with ACLs",
        steps: vec![
            // --- Create groups ---
            step(
                "Create group: engineering",
                &["--user", "admin@acme.com", "group", "create", "engineering", "--display-name", "Engineering"],
            ),
            step(
                "Create group: frontend",
                &["--user", "admin@acme.com", "group", "create", "frontend", "--display-name", "Frontend"],
            ),
            step(
                "Create group: backend",
                &["--user", "admin@acme.com", "group", "create", "backend", "--display-name", "Backend"],
            ),
            step(
                "Create group: devops",
                &["--user", "admin@acme.com", "group", "create", "devops", "--display-name", "DevOps"],
            ),
            step(
                "Create group: all-staff",
                &["--user", "admin@acme.com", "group", "create", "all-staff", "--display-name", "All Staff"],
            ),
            // --- Add user members to frontend ---
            step(
                "Add alice@acme.com to frontend",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "frontend",
                    "--member-type", "user",
                    "--member-value", "alice@acme.com",
                ],
            ),
            step(
                "Add bob@acme.com to frontend",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "frontend",
                    "--member-type", "user",
                    "--member-value", "bob@acme.com",
                ],
            ),
            // --- Add user members to backend ---
            step(
                "Add carol@acme.com to backend",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "backend",
                    "--member-type", "user",
                    "--member-value", "carol@acme.com",
                ],
            ),
            step(
                "Add dave@acme.com to backend",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "backend",
                    "--member-type", "user",
                    "--member-value", "dave@acme.com",
                ],
            ),
            // --- Add user member to devops ---
            step(
                "Add eve@acme.com to devops",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "devops",
                    "--member-type", "user",
                    "--member-value", "eve@acme.com",
                ],
            ),
            // --- Nest groups: frontend/backend/devops -> engineering -> all-staff ---
            step(
                "Add group 'frontend' as member of 'engineering'",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "engineering",
                    "--member-type", "group",
                    "--member-value", "frontend",
                ],
            ),
            step(
                "Add group 'backend' as member of 'engineering'",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "engineering",
                    "--member-type", "group",
                    "--member-value", "backend",
                ],
            ),
            step(
                "Add group 'devops' as member of 'engineering'",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "engineering",
                    "--member-type", "group",
                    "--member-value", "devops",
                ],
            ),
            step(
                "Add group 'engineering' as member of 'all-staff'",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "all-staff",
                    "--member-type", "group",
                    "--member-value", "engineering",
                ],
            ),
            // --- Query the hierarchy ---
            step(
                "List all groups",
                &["--user", "admin@acme.com", "group", "list"],
            ),
            step(
                "List members of engineering (3 nested group members)",
                &["--user", "admin@acme.com", "group", "list-members", "engineering"],
            ),
            step(
                "List members of frontend (2 user members)",
                &["--user", "admin@acme.com", "group", "list-members", "frontend"],
            ),
            step(
                "Resolve alice@acme.com groups (frontend -> engineering -> all-staff)",
                &["--user", "admin@acme.com", "group", "resolve-groups", "alice@acme.com"],
            ),
            step(
                "Check is-member: alice in all-staff (true via transitive membership)",
                &["--user", "admin@acme.com", "group", "is-member", "alice@acme.com", "all-staff"],
            ),
            step(
                "Check is-member: carol in frontend (false -- carol is in backend)",
                &["--user", "admin@acme.com", "group", "is-member", "carol@acme.com", "frontend"],
            ),
            // --- Integrate with ACLs ---
            step(
                "Create component: Infrastructure",
                &["component", "create", "Infrastructure", "--description", "Infrastructure services"],
            ),
            step(
                "Grant ADMIN_COMPONENTS to admin@acme.com on component 1",
                &[
                    "acl", "set-component", "1",
                    "--identity-type", "user",
                    "--identity-value", "admin@acme.com",
                    "--permissions", "ADMIN_COMPONENTS",
                ],
            ),
            step(
                "Set GROUP ACL: engineering gets VIEW_ISSUES,COMMENT_ON_ISSUES on component 1",
                &[
                    "acl", "set-component", "1",
                    "--identity-type", "group",
                    "--identity-value", "engineering",
                    "--permissions", "VIEW_ISSUES,COMMENT_ON_ISSUES",
                ],
            ),
            step(
                "View component 1 ACL (user + group entries)",
                &["acl", "get-component", "1"],
            ),
            step(
                "Check alice's effective permissions (VIEW+COMMENT via group membership)",
                &["acl", "check", "1", "--user", "alice@acme.com"],
            ),
            // --- More member ops ---
            step(
                "Add frank@acme.com to devops",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "devops",
                    "--member-type", "user",
                    "--member-value", "frank@acme.com",
                ],
            ),
            step(
                "Add grace@acme.com to devops",
                &[
                    "--user", "admin@acme.com",
                    "group", "add-member", "devops",
                    "--member-type", "user",
                    "--member-value", "grace@acme.com",
                ],
            ),
            step(
                "List devops members (3 user members)",
                &["--user", "admin@acme.com", "group", "list-members", "devops"],
            ),
            step(
                "Promote eve@acme.com to MANAGER in devops",
                &[
                    "--user", "admin@acme.com",
                    "group", "update-member-role", "devops",
                    "--member-type", "user",
                    "--member-value", "eve@acme.com",
                    "--role", "manager",
                ],
            ),
            step(
                "Update engineering display name to 'Engineering Division'",
                &["--user", "admin@acme.com", "group", "update", "engineering", "--display-name", "Engineering Division"],
            ),
            step(
                "Get engineering group (updated display name)",
                &["--user", "admin@acme.com", "group", "get", "engineering"],
            ),
            // --- Deletion preconditions ---
            step_fail(
                "Try to delete frontend while it is a member of engineering (expect failure)",
                &["--user", "admin@acme.com", "group", "delete", "frontend"],
            ),
            step(
                "Remove frontend from engineering first",
                &[
                    "--user", "admin@acme.com",
                    "group", "remove-member", "engineering",
                    "--member-type", "group",
                    "--member-value", "frontend",
                ],
            ),
            step(
                "Delete frontend (succeeds after removing membership)",
                &["--user", "admin@acme.com", "group", "delete", "frontend"],
            ),
            step(
                "List all groups (4 remaining)",
                &["--user", "admin@acme.com", "group", "list"],
            ),
        ],
    }
}
