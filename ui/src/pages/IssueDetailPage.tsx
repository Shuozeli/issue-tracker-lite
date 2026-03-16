import { useState } from "react";
import { useParams } from "react-router-dom";
import {
  Card, Descriptions, Tag, Spin, List, Button, Input, Select, Space, Divider,
  Typography, Modal, Timeline, Popconfirm, Tooltip, Alert,
} from "antd";
import {
  SendOutlined, EditOutlined, EyeInvisibleOutlined, HistoryOutlined,
  CheckOutlined, CloseOutlined,
} from "@ant-design/icons";
import { useSelector } from "react-redux";
import type { RootState } from "../store";
import {
  useGetIssueQuery,
  useUpdateIssueMutation,
  useListCommentsQuery,
  useCreateCommentMutation,
  useUpdateCommentMutation,
  useHideCommentMutation,
  useListCommentRevisionsQuery,
} from "../store/api";
import type { Status, Priority, Comment } from "../api/types";
import { priorityColor, statusColor, typeColor, formatTimestamp } from "../components/formatHelpers";
import { tid } from "../testIds";

const { Text } = Typography;

const statuses: Status[] = [
  "NEW", "ASSIGNED", "IN_PROGRESS", "INACTIVE",
  "FIXED", "FIXED_VERIFIED",
  "WONT_FIX_INFEASIBLE", "WONT_FIX_NOT_REPRODUCIBLE", "WONT_FIX_OBSOLETE", "WONT_FIX_INTENDED_BEHAVIOR",
  "DUPLICATE",
];
const priorities: Priority[] = ["P0", "P1", "P2", "P3", "P4"];

function RevisionHistory({ commentId }: { commentId: number }) {
  const { data, isLoading } = useListCommentRevisionsQuery(commentId);
  const revisions = data?.revisions ?? [];

  if (isLoading) return <Spin size="small" />;
  if (revisions.length === 0) return <Text type="secondary">No previous versions.</Text>;

  return (
    <Timeline
      items={revisions.map((rev) => ({
        children: (
          <div key={rev.revisionId}>
            <div>
              <Text strong>{rev.editedBy}</Text>{" "}
              <Text type="secondary">{formatTimestamp(rev.createTime)}</Text>
            </div>
            <div style={{ whiteSpace: "pre-wrap", background: "rgba(255,255,255,0.04)", padding: 8, borderRadius: 4, marginTop: 4 }}>
              {rev.body}
            </div>
          </div>
        ),
      }))}
    />
  );
}

function CommentItem({
  comment,
  currentUserId,
}: {
  comment: Comment;
  currentUserId: string;
}) {
  const [editing, setEditing] = useState(false);
  const [editBody, setEditBody] = useState(comment.body);
  const [showRevisions, setShowRevisions] = useState(false);
  const [updateComment] = useUpdateCommentMutation();
  const [hideComment] = useHideCommentMutation();

  const isAuthor = comment.author === currentUserId;

  const handleSaveEdit = async () => {
    if (!editBody.trim()) return;
    await updateComment({ commentId: comment.commentId, body: editBody });
    setEditing(false);
  };

  const handleCancelEdit = () => {
    setEditBody(comment.body);
    setEditing(false);
  };

  const handleHide = async () => {
    await hideComment({ commentId: comment.commentId });
  };

  return (
    <List.Item
      style={comment.hidden ? { opacity: 0.5 } : undefined}
      actions={
        comment.hidden
          ? []
          : [
              ...(isAuthor && !editing
                ? [
                    <Tooltip title="Edit" key="edit">
                      <Button
                        data-testid={tid.issueDetail.commentEditBtn}
                        type="text"
                        size="small"
                        icon={<EditOutlined />}
                        onClick={() => setEditing(true)}
                      />
                    </Tooltip>,
                  ]
                : []),
              ...(comment.revisionCount > 0
                ? [
                    <Tooltip title={`${comment.revisionCount} revision(s)`} key="history">
                      <Button
                        data-testid={tid.issueDetail.commentHistoryBtn}
                        type="text"
                        size="small"
                        icon={<HistoryOutlined />}
                        onClick={() => setShowRevisions(true)}
                      />
                    </Tooltip>,
                  ]
                : []),
              ...(!comment.isDescription
                ? [
                    <Popconfirm
                      key="hide"
                      title="Remove this comment?"
                      description="The content will be permanently redacted. Previous versions are kept for audit."
                      onConfirm={handleHide}
                      okText="Remove"
                      cancelText="Cancel"
                    >
                      <Tooltip title="Remove">
                        <Button data-testid={tid.issueDetail.commentHideBtn} type="text" size="small" danger icon={<EyeInvisibleOutlined />} />
                      </Tooltip>
                    </Popconfirm>,
                  ]
                : []),
            ]
      }
    >
      <List.Item.Meta
        title={
          <Space>
            <Text strong>{comment.author || "anonymous"}</Text>
            <Text type="secondary">{formatTimestamp(comment.createTime)}</Text>
            {comment.modifyTime && <Text type="secondary">(edited)</Text>}
            {comment.hidden && <Tag color="red">removed</Tag>}
          </Space>
        }
        description={
          comment.hidden ? (
            <Alert
              message="This comment has been removed by a moderator."
              type="warning"
              showIcon={false}
              style={{ marginTop: 4 }}
            />
          ) : editing ? (
            <Space direction="vertical" style={{ width: "100%", marginTop: 4 }}>
              <Input.TextArea
                data-testid={tid.issueDetail.commentEditTextarea}
                value={editBody}
                onChange={(e) => setEditBody(e.target.value)}
                rows={3}
                autoFocus
              />
              <Space>
                <Button data-testid={tid.issueDetail.commentEditSave} type="primary" size="small" icon={<CheckOutlined />} onClick={handleSaveEdit}>
                  Save
                </Button>
                <Button data-testid={tid.issueDetail.commentEditCancel} size="small" icon={<CloseOutlined />} onClick={handleCancelEdit}>
                  Cancel
                </Button>
              </Space>
            </Space>
          ) : (
            <div style={{ whiteSpace: "pre-wrap" }}>{comment.body}</div>
          )
        }
      />
      <Modal
        title="Revision History"
        open={showRevisions}
        onCancel={() => setShowRevisions(false)}
        footer={null}
        width={600}
      >
        <RevisionHistory commentId={comment.commentId} />
      </Modal>
    </List.Item>
  );
}

export default function IssueDetailPage() {
  const { id } = useParams<{ id: string }>();
  const issueId = Number(id);
  const { data: issue, isLoading } = useGetIssueQuery(issueId);
  const { data: commentsData } = useListCommentsQuery(issueId);
  const [updateIssue] = useUpdateIssueMutation();
  const [createComment] = useCreateCommentMutation();
  const [commentBody, setCommentBody] = useState("");
  const userId = useSelector((state: RootState) => state.auth.userId) ?? "";

  if (isLoading || !issue) return <Spin size="large" />;

  const handleStatusChange = (status: Status) => {
    updateIssue({ id: issueId, status });
  };

  const handlePriorityChange = (priority: Priority) => {
    updateIssue({ id: issueId, priority });
  };

  const handleAddComment = async () => {
    if (!commentBody.trim()) return;
    await createComment({
      issueId,
      body: commentBody,
      author: userId,
    });
    setCommentBody("");
  };

  const comments = commentsData?.comments ?? [];

  return (
    <div>
      <Card title={`Issue #${issue.issueId}: ${issue.title}`} style={{ marginBottom: 16 }}>
        <Descriptions column={2} bordered size="small">
          <Descriptions.Item label="Status">
            <Select data-testid={tid.issueDetail.selectStatus} value={issue.status} onChange={handleStatusChange} style={{ width: 200 }} size="small">
              {statuses.map((s) => (
                <Select.Option key={s} value={s}>
                  <Tag color={statusColor(s)}>{s}</Tag>
                </Select.Option>
              ))}
            </Select>
          </Descriptions.Item>
          <Descriptions.Item label="Priority">
            <Select data-testid={tid.issueDetail.selectPriority} value={issue.priority} onChange={handlePriorityChange} style={{ width: 100 }} size="small">
              {priorities.map((p) => (
                <Select.Option key={p} value={p}>
                  <Tag color={priorityColor(p)}>{p}</Tag>
                </Select.Option>
              ))}
            </Select>
          </Descriptions.Item>
          <Descriptions.Item label="Type">
            <Tag color={typeColor(issue.type)}>{issue.type}</Tag>
          </Descriptions.Item>
          <Descriptions.Item label="Severity">
            {issue.severity !== "SEVERITY_UNSPECIFIED" ? issue.severity : "-"}
          </Descriptions.Item>
          <Descriptions.Item label="Assignee">{issue.assignee || "-"}</Descriptions.Item>
          <Descriptions.Item label="Reporter">{issue.reporter || "-"}</Descriptions.Item>
          <Descriptions.Item label="Component ID">{issue.componentId}</Descriptions.Item>
          <Descriptions.Item label="Created">{formatTimestamp(issue.createTime)}</Descriptions.Item>
          <Descriptions.Item label="Duplicates">{issue.duplicateCount}</Descriptions.Item>
          <Descriptions.Item label="Votes">{issue.voteCount}</Descriptions.Item>
        </Descriptions>
        {issue.description && (
          <>
            <Divider orientation="left" plain>Description</Divider>
            <Text>{issue.description}</Text>
          </>
        )}
      </Card>

      <Card title={`Comments (${comments.length})`}>
        <List
          dataSource={comments}
          renderItem={(comment) => (
            <CommentItem
              key={comment.commentId}
              comment={comment}
              currentUserId={userId}
            />
          )}
          locale={{ emptyText: "No comments yet" }}
        />
        <Divider />
        <Space.Compact style={{ width: "100%" }}>
          <Input
            value={userId}
            disabled
            style={{ width: 200 }}
            prefix={<Text type="secondary">As:</Text>}
          />
          <Input
            data-testid={tid.issueDetail.commentInput}
            placeholder="Write a comment..."
            value={commentBody}
            onChange={(e) => setCommentBody(e.target.value)}
            onPressEnter={handleAddComment}
            style={{ flex: 1 }}
          />
          <Button data-testid={tid.issueDetail.commentSend} type="primary" icon={<SendOutlined />} onClick={handleAddComment}>
            Send
          </Button>
        </Space.Compact>
      </Card>
    </div>
  );
}
