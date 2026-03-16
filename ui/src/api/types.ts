// Types matching proto definitions (camelCase from proto-loader)

export type IssueType =
  | "BUG"
  | "FEATURE_REQUEST"
  | "CUSTOMER_ISSUE"
  | "INTERNAL_CLEANUP"
  | "PROCESS"
  | "VULNERABILITY"
  | "PRIVACY_ISSUE"
  | "PROGRAM"
  | "PROJECT"
  | "FEATURE"
  | "MILESTONE"
  | "EPIC"
  | "STORY"
  | "TASK"
  | "ISSUE_TYPE_UNSPECIFIED";

export type Priority = "P0" | "P1" | "P2" | "P3" | "P4" | "PRIORITY_UNSPECIFIED";

export type Severity = "S0" | "S1" | "S2" | "S3" | "S4" | "SEVERITY_UNSPECIFIED";

export type Status =
  | "NEW"
  | "ASSIGNED"
  | "IN_PROGRESS"
  | "INACTIVE"
  | "FIXED"
  | "FIXED_VERIFIED"
  | "WONT_FIX_INFEASIBLE"
  | "WONT_FIX_NOT_REPRODUCIBLE"
  | "WONT_FIX_OBSOLETE"
  | "WONT_FIX_INTENDED_BEHAVIOR"
  | "DUPLICATE"
  | "STATUS_UNSPECIFIED";

export interface Timestamp {
  seconds: number;
  nanos: number;
}

export interface Component {
  componentId: number;
  name: string;
  description: string;
  parentId: number | null;
  expandedAccessEnabled: boolean;
  editableCommentsEnabled: boolean;
  createTime: Timestamp | null;
  updateTime: Timestamp | null;
  childCount: number;
}

export interface Issue {
  issueId: number;
  title: string;
  description: string;
  status: Status;
  priority: Priority;
  severity: Severity;
  type: IssueType;
  componentId: number;
  assignee: string;
  reporter: string;
  verifier: string;
  createTime: Timestamp | null;
  modifyTime: Timestamp | null;
  resolveTime: Timestamp | null;
  verifyTime: Timestamp | null;
  voteCount: number;
  duplicateCount: number;
  foundIn: string;
  targetedTo: string;
  verifiedIn: string;
  inProd: boolean;
  archived: boolean;
  accessLevel: string;
}

export interface Comment {
  commentId: number;
  issueId: number;
  author: string;
  body: string;
  isDescription: boolean;
  createTime: Timestamp | null;
  modifyTime: Timestamp | null;
  hidden: boolean;
  hiddenBy: string;
  hiddenTime: Timestamp | null;
  revisionCount: number;
}

export interface CommentRevision {
  revisionId: number;
  commentId: number;
  body: string;
  editedBy: string;
  createTime: Timestamp | null;
}

export interface Hotlist {
  hotlistId: number;
  name: string;
  description: string;
  owner: string;
  archived: boolean;
  createTime: Timestamp | null;
  modifyTime: Timestamp | null;
  issueCount: number;
}

export interface HotlistIssue {
  hotlistId: number;
  issueId: number;
  position: number;
  addTime: Timestamp | null;
  addedBy: string;
}

export interface Event {
  eventId: number;
  eventTime: Timestamp | null;
  eventType: string;
  actor: string;
  entityType: string;
  entityId: number;
  payload: string;
}

// Request types

export interface CreateComponentRequest {
  name: string;
  description?: string;
  parentId?: number;
}

export interface CreateIssueRequest {
  componentId: number;
  title: string;
  description?: string;
  type?: IssueType;
  priority?: Priority;
  severity?: Severity;
  reporter?: string;
  assignee?: string;
}

export interface UpdateIssueRequest {
  status?: Status;
  priority?: Priority;
  severity?: Severity;
  assignee?: string;
  title?: string;
  description?: string;
}

export interface CreateCommentRequest {
  body: string;
  author?: string;
}

export interface UpdateCommentRequest {
  commentId: number;
  body: string;
}

export interface CreateHotlistRequest {
  name: string;
  description?: string;
  owner?: string;
}
