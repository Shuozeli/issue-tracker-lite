---
trigger: always_on
---

## Doc Freshness Rule

Every markdown document created or updated by the agent **MUST** include a freshness line at the very top of the file (after any YAML frontmatter), in exactly this format:

```
<!-- agent-updated: YYYY-MM-DDTHH:MM:SSZ -->
```

* The timestamp **MUST** use UTC (Z suffix) and ISO 8601 format.
* The agent **MUST** update this line every time it writes to or rewrites the document.
* The agent **MUST** check this line when auditing docs to determine if the content is stale relative to known code changes.
* If no `agent-updated` line is present, the document is considered **unreviewed**.


#### 1. `README.md` (Entry Point)

* **Purpose:** The landing page for the sub-project. It answers "What is this?" and "How do I run it?"
* **Format:**
* **Title:** Project Name
* **Description:** One-paragraph summary of functionality.
* **Dependencies:** List of required libs/tools.
* **Quick Start:** Commands to install, build, and run locally.



#### 2. `tasks.md` (Roadmap & Status)

* **Purpose:** Tracks immediate to-dos and technical debt.
* **Format:** Markdown checklist.
* `- [x] Completed Item (YYYY-MM-DD)`
* `- [ ] Pending Item`
* `- [ ] **Bug**: Description of issue`



#### 3. `API.md` (Interface Reference)

* **Purpose:** details public interfaces, endpoints, or exported functions.
* **Format:**
* **Endpoint/Method Signature**
* **Inputs:** Parameters and types.
* **Outputs:** Return values/JSON response examples.
* **Errors:** Potential error codes thrown.



#### 4. `CHANGELOG.md` (History)

* **Purpose:** A chronological log of changes for other developers.
* **Format:** Reverse chronological order (newest first).
* `## [Version] - YYYY-MM-DD`
* `### Added` / `### Changed` / `### Fixed`



#### 5. `ADR/` (Architecture Decision Records)

* **Purpose:** A folder containing numbered files (e.g., `001-use-postgres.md`) recording significant architectural decisions.
* **Format:**
* **Title:** Short imperative title.
* **Status:** Proposed / Accepted / Deprecated.
* **Context:** What was the problem?
* **Decision:** What did we choose?
* **Consequences:** What are the pros/cons of this choice?



#### 6. `architecture.md` (System Design)

* **Purpose:** Explains the high-level design and data flow.
* **Format:**
* **Diagrams:** (Use Mermaid.js syntax if supported, otherwise text description).
* **Key Components:** List of modules and their responsibilities.
* **Data Flow:** How data moves through the system.
