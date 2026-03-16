use super::{step, step_assert, step_fail, Pipeline};

pub fn pipeline() -> Pipeline {
    Pipeline {
        name: "full-lifecycle",
        summary: "Complete issue lifecycle from creation through verification, plus duplicates",
        steps: vec![
            step_assert(
                "Create component: Payments",
                &["--user", "admin@demo.com", "component", "create", "Payments", "--description", "Payment processing service"],
                &["Payments"],
            ),
            step(
                "Grant admin on component 1",
                &["--user", "admin@demo.com", "acl", "set-component", "1",
                  "--identity-type", "user", "--identity-value", "admin@demo.com",
                  "--permissions", "ADMIN_COMPONENTS"],
            ),
            step_assert(
                "Report a bug: Payment fails for amounts > $10,000 (status: NEW)",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Payment fails for amounts over $10,000",
                    "--description", "Transactions above $10,000 return a 422 error. Likely an integer overflow in the amount field.",
                    "--priority", "P0",
                    "--type", "BUG",
                    "--severity", "S0",
                    "--reporter", "support@example.com",
                ],
                &["Payment fails for amounts over $10,000"],
            ),
            step_assert(
                "Assign to alice (auto-transitions to ASSIGNED)",
                &["--user", "admin@demo.com", "issue", "update", "1", "--assignee", "alice@example.com"],
                &["ASSIGNED"],
            ),
            step_assert(
                "Alice starts investigating (IN_PROGRESS)",
                &["--user", "admin@demo.com", "issue", "update", "1", "--status", "IN_PROGRESS"],
                &["IN_PROGRESS"],
            ),
            step_assert(
                "Alice adds investigation findings",
                &[
                    "--user", "admin@demo.com",
                    "comment", "add", "1",
                    "--body", "Root cause found: amount stored as i32, overflows at 2^31 cents ($21,474,836.47). Need to migrate to i64.",
                    "--author", "alice@example.com",
                ],
                &["Root cause found"],
            ),
            step_assert(
                "A blocker is discovered: need DB migration first",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Migrate payment_amount column from INT to BIGINT",
                    "--priority", "P0",
                    "--type", "TASK",
                    "--assignee", "dba@example.com",
                ],
                &["Migrate payment_amount column from INT to BIGINT"],
            ),
            step(
                "Mark the DB migration as blocking the original bug fix",
                &["--user", "admin@demo.com", "issue", "block", "2", "1"],
            ),
            step_assert(
                "DBA completes the migration",
                &["--user", "admin@demo.com", "issue", "update", "2", "--status", "FIXED"],
                &["FIXED"],
            ),
            step_assert(
                "Alice adds fix comment on original bug",
                &[
                    "--user", "admin@demo.com",
                    "comment", "add", "1",
                    "--body", "DB migration complete. Changed amount type to i64 in application layer. All existing tests pass. Added regression test for $50,000 payment.",
                    "--author", "alice@example.com",
                ],
                &["DB migration complete"],
            ),
            step_assert(
                "Alice marks the bug as FIXED",
                &["--user", "admin@demo.com", "issue", "update", "1", "--status", "FIXED"],
                &["FIXED"],
            ),
            step_assert(
                "QA verifier adds verification comment",
                &[
                    "--user", "admin@demo.com",
                    "comment", "add", "1",
                    "--body", "Verified: $10,000, $50,000, and $100,000 payments all succeed in staging. Approving fix.",
                    "--author", "qa@example.com",
                ],
                &["Verified"],
            ),
            step_assert(
                "QA marks as FIXED_VERIFIED",
                &["--user", "admin@demo.com", "issue", "update", "1", "--status", "FIXED_VERIFIED"],
                &["FIXED_VERIFIED"],
            ),
            step_assert(
                "View the fully resolved issue",
                &["--user", "admin@demo.com", "issue", "get", "1"],
                &["FIXED_VERIFIED"],
            ),
            step_assert(
                "View all comments on the issue",
                &["--user", "admin@demo.com", "comment", "list", "1"],
                &["Root cause found"],
            ),
            step_assert(
                "Someone files a duplicate bug",
                &[
                    "--user", "admin@demo.com",
                    "issue", "create",
                    "--component", "1",
                    "--title", "Large payments rejected with error 422",
                    "--priority", "P0",
                    "--type", "BUG",
                    "--reporter", "sales@example.com",
                ],
                &["Large payments rejected with error 422"],
            ),
            step_assert(
                "Mark it as a duplicate of the original (issue 3 duplicates issue 1)",
                &["--user", "admin@demo.com", "issue", "duplicate", "3", "--of", "1"],
                &["DUPLICATE"],
            ),
            step_assert(
                "View the original -- duplicate_count should be 1",
                &["--user", "admin@demo.com", "issue", "get", "1"],
                &["FIXED_VERIFIED"],
            ),
            step_fail(
                "Attempt an invalid transition: reopen FIXED_VERIFIED -> IN_PROGRESS",
                &["--user", "admin@demo.com", "issue", "update", "1", "--status", "IN_PROGRESS"],
            ),
            step_assert(
                "Check event log for issue 1 (full audit trail)",
                &["--user", "admin@demo.com", "events", "--entity-type", "Issue", "--entity-id", "1"],
                &["ISSUE"],
            ),
        ],
    }
}
