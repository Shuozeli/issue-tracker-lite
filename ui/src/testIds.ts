/**
 * Centralized data-testid constants for all interactive UI elements.
 *
 * Convention: {page}_{section}_{component}
 *
 * Used by:
 *  - UI components: <Input data-testid={tid.login.email} />
 *  - Demo runner:   { ui_fill: tid.login.email, value: "user@example.com" }
 *  - E2E tests:     page.locator(`[data-testid="${tid.login.email}"]`)
 */
export const tid = {
  login: {
    email: "login_email",
    submit: "login_submit",
  },
  nav: {
    dashboard: "nav_dashboard",
    issues: "nav_issues",
    components: "nav_components",
    hotlists: "nav_hotlists",
    search: "nav_search",
    events: "nav_events",
  },
  header: {
    userId: "header_user_id",
    signOut: "header_sign_out",
  },
  dashboard: {
    statComponents: "dashboard_stat_components",
    statOpenIssues: "dashboard_stat_open_issues",
    statClosed: "dashboard_stat_closed",
    statP0Open: "dashboard_stat_p0_open",
    recentTable: "dashboard_recent_table",
  },
  components: {
    createBtn: "components_create_btn",
    table: "components_table",
    inputName: "components_create_name",
    inputDescription: "components_create_description",
    inputParentId: "components_create_parent_id",
  },
  issues: {
    createBtn: "issues_create_btn",
    table: "issues_table",
    selectComponent: "issues_create_component",
    inputTitle: "issues_create_title",
    inputDescription: "issues_create_description",
    selectType: "issues_create_type",
    selectPriority: "issues_create_priority",
    selectSeverity: "issues_create_severity",
    inputAssignee: "issues_create_assignee",
    inputReporter: "issues_create_reporter",
  },
  issueDetail: {
    selectStatus: "issue_detail_status",
    selectPriority: "issue_detail_priority",
    commentInput: "issue_detail_comment_input",
    commentSend: "issue_detail_comment_send",
    commentEditBtn: "comment_edit_btn",
    commentEditTextarea: "comment_edit_textarea",
    commentEditSave: "comment_edit_save",
    commentEditCancel: "comment_edit_cancel",
    commentHistoryBtn: "comment_history_btn",
    commentHideBtn: "comment_hide_btn",
  },
  hotlists: {
    createBtn: "hotlists_create_btn",
    table: "hotlists_table",
    inputName: "hotlists_create_name",
    inputDescription: "hotlists_create_description",
    inputOwner: "hotlists_create_owner",
  },
  search: {
    input: "search_input",
    submitBtn: "search_submit",
    resultsTable: "search_results_table",
  },
  events: {
    table: "events_table",
  },
} as const;
