<!-- agent-updated: 2026-03-15T20:00:00Z -->

# Database Schema -- Reference Design

SQLite schema for the Issue Tracker rebuild, managed by Quiver ORM (`schema.quiver`). All interactions must be wrapped in transactions (including reads, per project rules).

---

## Core Tables

### components

```sql
CREATE TABLE components (
  component_id BIGSERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  parent_component_id BIGINT REFERENCES components(component_id),
  expanded_access_enabled BOOLEAN NOT NULL DEFAULT TRUE,
  editable_comments_enabled BOOLEAN NOT NULL DEFAULT FALSE,
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modify_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_components_parent ON components(parent_component_id);
```

### component_acl

```sql
CREATE TABLE component_acl (
  component_id BIGINT NOT NULL REFERENCES components(component_id) ON DELETE CASCADE,
  identity_type TEXT NOT NULL CHECK (identity_type IN ('USER', 'GROUP', 'PUBLIC')),
  identity_value TEXT NOT NULL,
  permissions TEXT[] NOT NULL,
  PRIMARY KEY (component_id, identity_type, identity_value)
);
```

### custom_field_definitions

```sql
CREATE TABLE custom_field_definitions (
  custom_field_id BIGSERIAL PRIMARY KEY,
  component_id BIGINT NOT NULL REFERENCES components(component_id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  field_type TEXT NOT NULL CHECK (field_type IN ('TEXT', 'NUMBER', 'ENUM', 'DATE', 'USER', 'BOOLEAN')),
  enum_values TEXT[],
  required BOOLEAN NOT NULL DEFAULT FALSE,
  hidden BOOLEAN NOT NULL DEFAULT FALSE,
  edit_significance TEXT NOT NULL DEFAULT 'MINOR' CHECK (edit_significance IN ('MAJOR', 'MINOR', 'SILENT'))
);

CREATE INDEX idx_custom_fields_component ON custom_field_definitions(component_id);
```

### templates

```sql
CREATE TABLE templates (
  template_id BIGSERIAL PRIMARY KEY,
  component_id BIGINT NOT NULL REFERENCES components(component_id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  is_default BOOLEAN NOT NULL DEFAULT FALSE,
  is_inherited BOOLEAN NOT NULL DEFAULT FALSE,
  field_defaults JSONB NOT NULL DEFAULT '{}',
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modify_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_templates_component ON templates(component_id);
-- Enforce exactly one default template per component
CREATE UNIQUE INDEX idx_templates_default ON templates(component_id) WHERE is_default = TRUE;
```

---

### issues

```sql
CREATE TABLE issues (
  issue_id BIGSERIAL PRIMARY KEY,
  component_id BIGINT NOT NULL REFERENCES components(component_id),
  title TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'NEW'
    CHECK (status IN (
      'NEW', 'ASSIGNED', 'IN_PROGRESS', 'INACTIVE',
      'FIXED', 'FIXED_VERIFIED',
      'WONT_FIX_INFEASIBLE', 'WONT_FIX_NOT_REPRODUCIBLE',
      'WONT_FIX_OBSOLETE', 'WONT_FIX_INTENDED_BEHAVIOR',
      'DUPLICATE'
    )),
  priority TEXT NOT NULL DEFAULT 'P2'
    CHECK (priority IN ('P0', 'P1', 'P2', 'P3', 'P4')),
  severity TEXT NOT NULL DEFAULT 'S2'
    CHECK (severity IN ('S0', 'S1', 'S2', 'S3', 'S4')),
  type TEXT NOT NULL DEFAULT 'BUG'
    CHECK (type IN (
      'BUG', 'FEATURE_REQUEST', 'CUSTOMER_ISSUE', 'INTERNAL_CLEANUP',
      'PROCESS', 'VULNERABILITY', 'PRIVACY_ISSUE',
      'PROGRAM', 'PROJECT', 'FEATURE', 'MILESTONE', 'EPIC', 'STORY', 'TASK'
    )),
  access_level TEXT NOT NULL DEFAULT 'DEFAULT'
    CHECK (access_level IN (
      'DEFAULT', 'LIMITED_COMMENTING', 'LIMITED_VISIBILITY', 'LIMITED_VISIBILITY_GOOGLE'
    )),
  assignee TEXT,
  reporter TEXT NOT NULL,
  verifier TEXT,
  found_in TEXT NOT NULL DEFAULT '',
  targeted_to TEXT NOT NULL DEFAULT '',
  verified_in TEXT NOT NULL DEFAULT '',
  in_prod BOOLEAN NOT NULL DEFAULT FALSE,
  archived BOOLEAN NOT NULL DEFAULT FALSE,
  vote_count INT NOT NULL DEFAULT 0,
  duplicate_count INT NOT NULL DEFAULT 0,
  duplicate_of BIGINT REFERENCES issues(issue_id),
  status_update TEXT NOT NULL DEFAULT '',
  estimated_effort TEXT NOT NULL DEFAULT '',
  start_date DATE,
  end_date DATE,
  custom_fields JSONB NOT NULL DEFAULT '{}',
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modify_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  resolve_time TIMESTAMPTZ,
  verify_time TIMESTAMPTZ
);

CREATE INDEX idx_issues_component ON issues(component_id);
CREATE INDEX idx_issues_assignee ON issues(assignee) WHERE assignee IS NOT NULL;
CREATE INDEX idx_issues_reporter ON issues(reporter);
CREATE INDEX idx_issues_status ON issues(status);
CREATE INDEX idx_issues_priority ON issues(priority);
CREATE INDEX idx_issues_type ON issues(type);
CREATE INDEX idx_issues_create_time ON issues(create_time);
CREATE INDEX idx_issues_modify_time ON issues(modify_time);
CREATE INDEX idx_issues_duplicate_of ON issues(duplicate_of) WHERE duplicate_of IS NOT NULL;

-- Full-text search index
ALTER TABLE issues ADD COLUMN search_vector tsvector;
CREATE INDEX idx_issues_search ON issues USING GIN(search_vector);
```

### issue_collaborators

```sql
CREATE TABLE issue_collaborators (
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  user_email TEXT NOT NULL,
  PRIMARY KEY (issue_id, user_email)
);
```

### issue_cc

```sql
CREATE TABLE issue_cc (
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  identity_type TEXT NOT NULL CHECK (identity_type IN ('USER', 'GROUP')),
  identity_value TEXT NOT NULL,
  PRIMARY KEY (issue_id, identity_type, identity_value)
);
```

---

### comments

```sql
CREATE TABLE comments (
  comment_id BIGSERIAL PRIMARY KEY,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  author TEXT NOT NULL,
  body TEXT NOT NULL,
  restriction_level TEXT NOT NULL DEFAULT 'UNRESTRICTED'
    CHECK (restriction_level IN ('UNRESTRICTED', 'RESTRICTED', 'RESTRICTED_PLUS')),
  is_description BOOLEAN NOT NULL DEFAULT FALSE,
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modify_time TIMESTAMPTZ
);

CREATE INDEX idx_comments_issue ON comments(issue_id);
CREATE INDEX idx_comments_author ON comments(author);
```

### attachments

```sql
CREATE TABLE attachments (
  attachment_id BIGSERIAL PRIMARY KEY,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  comment_id BIGINT REFERENCES comments(comment_id),
  filename TEXT NOT NULL,
  content_type TEXT NOT NULL,
  size_bytes BIGINT NOT NULL,
  storage_path TEXT NOT NULL,
  uploader TEXT NOT NULL,
  restriction_level TEXT NOT NULL DEFAULT 'UNRESTRICTED'
    CHECK (restriction_level IN ('UNRESTRICTED', 'RESTRICTED', 'RESTRICTED_PLUS')),
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_attachments_issue ON attachments(issue_id);
```

---

## Relationship Tables

### issue_parents (N:N parent-child)

```sql
CREATE TABLE issue_parents (
  parent_issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  child_issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  position INT NOT NULL DEFAULT 0,
  PRIMARY KEY (parent_issue_id, child_issue_id),
  CHECK (parent_issue_id != child_issue_id)
);

CREATE INDEX idx_issue_parents_child ON issue_parents(child_issue_id);
```

### issue_blocking (N:N blocking)

```sql
CREATE TABLE issue_blocking (
  blocking_issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  blocked_issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  PRIMARY KEY (blocking_issue_id, blocked_issue_id),
  CHECK (blocking_issue_id != blocked_issue_id)
);

CREATE INDEX idx_issue_blocking_blocked ON issue_blocking(blocked_issue_id);
```

---

## Hotlists

```sql
CREATE TABLE hotlists (
  hotlist_id BIGSERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  owner TEXT NOT NULL,
  archived BOOLEAN NOT NULL DEFAULT FALSE,
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modify_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE hotlist_acl (
  hotlist_id BIGINT NOT NULL REFERENCES hotlists(hotlist_id) ON DELETE CASCADE,
  identity_type TEXT NOT NULL CHECK (identity_type IN ('USER', 'GROUP', 'PUBLIC')),
  identity_value TEXT NOT NULL,
  permission TEXT NOT NULL CHECK (permission IN ('HOTLIST_ADMIN', 'HOTLIST_VIEW_APPEND', 'HOTLIST_VIEW')),
  PRIMARY KEY (hotlist_id, identity_type, identity_value)
);

CREATE TABLE hotlist_issues (
  hotlist_id BIGINT NOT NULL REFERENCES hotlists(hotlist_id) ON DELETE CASCADE,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  position INT NOT NULL DEFAULT 0,
  add_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  added_by TEXT NOT NULL,
  PRIMARY KEY (hotlist_id, issue_id)
);

CREATE INDEX idx_hotlist_issues_issue ON hotlist_issues(issue_id);
```

---

## Saved Searches

```sql
CREATE TABLE saved_searches (
  saved_search_id BIGSERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  query TEXT NOT NULL,
  owner TEXT NOT NULL,
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modify_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE saved_search_acl (
  saved_search_id BIGINT NOT NULL REFERENCES saved_searches(saved_search_id) ON DELETE CASCADE,
  identity_type TEXT NOT NULL CHECK (identity_type IN ('USER', 'GROUP', 'PUBLIC')),
  identity_value TEXT NOT NULL,
  permission TEXT NOT NULL CHECK (permission IN ('SEARCH_ADMIN', 'SEARCH_VIEW_EXECUTE')),
  PRIMARY KEY (saved_search_id, identity_type, identity_value)
);
```

---

## Bookmark Groups

```sql
CREATE TABLE bookmark_groups (
  bookmark_group_id BIGSERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  archived BOOLEAN NOT NULL DEFAULT FALSE,
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modify_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE bookmark_group_acl (
  bookmark_group_id BIGINT NOT NULL REFERENCES bookmark_groups(bookmark_group_id) ON DELETE CASCADE,
  identity_type TEXT NOT NULL CHECK (identity_type IN ('USER', 'GROUP', 'PUBLIC')),
  identity_value TEXT NOT NULL,
  permission TEXT NOT NULL CHECK (permission IN ('BOOKMARK_ADMIN', 'BOOKMARK_VIEW')),
  PRIMARY KEY (bookmark_group_id, identity_type, identity_value)
);

CREATE TABLE bookmark_group_items (
  bookmark_group_id BIGINT NOT NULL REFERENCES bookmark_groups(bookmark_group_id) ON DELETE CASCADE,
  position INT NOT NULL DEFAULT 0,
  item_type TEXT NOT NULL CHECK (item_type IN ('HOTLIST', 'SAVED_SEARCH')),
  item_id BIGINT NOT NULL,
  PRIMARY KEY (bookmark_group_id, item_type, item_id)
);
```

---

## Trackers

```sql
CREATE TABLE trackers (
  tracker_id BIGSERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  domain_url TEXT,
  logo_url TEXT NOT NULL DEFAULT '',
  color_theme TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT '',
  trusted_groups TEXT[] NOT NULL DEFAULT '{}',
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  modify_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE tracker_components (
  tracker_id BIGINT NOT NULL REFERENCES trackers(tracker_id) ON DELETE CASCADE,
  component_id BIGINT NOT NULL REFERENCES components(component_id) ON DELETE CASCADE,
  PRIMARY KEY (tracker_id, component_id)
);
```

---

## User Tables

```sql
CREATE TABLE user_settings (
  user_email TEXT PRIMARY KEY,
  homepage TEXT NOT NULL DEFAULT 'assigned_to_me',
  date_format TEXT NOT NULL DEFAULT 'MMM_DD_YYYY',
  time_format TEXT NOT NULL DEFAULT 'HH_MM_AMPM',
  timezone TEXT NOT NULL DEFAULT 'LOCAL',
  keyboard_shortcuts_enabled BOOLEAN NOT NULL DEFAULT TRUE,
  force_plain_text_comments BOOLEAN NOT NULL DEFAULT FALSE,
  force_code_font_comments BOOLEAN NOT NULL DEFAULT FALSE,
  exclude_own_edits BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE user_notification_settings (
  user_email TEXT NOT NULL,
  role TEXT NOT NULL CHECK (role IN ('ASSIGNEE', 'REPORTER', 'VERIFIER', 'COLLABORATOR', 'CC', 'STARRED')),
  level TEXT NOT NULL CHECK (level IN ('ALL_UPDATES', 'MAJOR_UPDATES', 'CLOSURE_ONLY', 'MINIMAL')),
  PRIMARY KEY (user_email, role)
);

CREATE TABLE issue_notification_overrides (
  user_email TEXT NOT NULL,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  level TEXT NOT NULL CHECK (level IN ('ALL_UPDATES', 'MAJOR_UPDATES', 'CLOSURE_ONLY', 'MINIMAL', 'NONE')),
  PRIMARY KEY (user_email, issue_id)
);

CREATE TABLE issue_read_status (
  user_email TEXT NOT NULL,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  is_read BOOLEAN NOT NULL DEFAULT FALSE,
  last_read_time TIMESTAMPTZ,
  PRIMARY KEY (user_email, issue_id)
);

CREATE TABLE issue_stars (
  user_email TEXT NOT NULL,
  issue_id BIGINT NOT NULL REFERENCES issues(issue_id) ON DELETE CASCADE,
  create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (user_email, issue_id)
);
```

---

## Event Log

Per project rules, the system must have an event log for debugging.

```sql
CREATE TABLE event_log (
  event_id BIGSERIAL PRIMARY KEY,
  event_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  event_type TEXT NOT NULL,
  actor TEXT NOT NULL,
  entity_type TEXT NOT NULL,
  entity_id BIGINT NOT NULL,
  payload JSONB NOT NULL DEFAULT '{}'
);

CREATE INDEX idx_event_log_time ON event_log(event_time);
CREATE INDEX idx_event_log_entity ON event_log(entity_type, entity_id);
CREATE INDEX idx_event_log_type ON event_log(event_type);
```
