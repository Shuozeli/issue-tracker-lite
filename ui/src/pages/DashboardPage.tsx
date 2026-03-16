import { Card, Col, Row, Statistic, Spin, Table, Tag } from "antd";
import { BugOutlined, AppstoreOutlined, CheckCircleOutlined, ClockCircleOutlined } from "@ant-design/icons";
import { useListComponentsQuery, useListIssuesQuery } from "../store/api";
import type { Issue } from "../api/types";
import { priorityColor, statusColor, formatTimestamp } from "../components/formatHelpers";
import { tid } from "../testIds";

const CLOSED_STATUSES = ["FIXED", "FIXED_VERIFIED", "WONT_FIX_INFEASIBLE", "WONT_FIX_NOT_REPRODUCIBLE", "WONT_FIX_OBSOLETE", "WONT_FIX_INTENDED_BEHAVIOR", "DUPLICATE"];

export default function DashboardPage() {
  const { data: compData, isLoading: compLoading } = useListComponentsQuery();
  const { data: issueData, isLoading: issueLoading } = useListIssuesQuery();

  if (compLoading || issueLoading) return <Spin size="large" />;

  const components = compData?.components ?? [];
  const issues = issueData?.issues ?? [];
  const openIssues = issues.filter((i) => !CLOSED_STATUSES.includes(i.status));
  const closedIssues = issues.filter((i) => CLOSED_STATUSES.includes(i.status));
  const p0Open = openIssues.filter((i) => i.priority === "P0");

  const recentColumns = [
    { title: "ID", dataIndex: "issueId", key: "issueId", width: 60 },
    { title: "Title", dataIndex: "title", key: "title", ellipsis: true },
    {
      title: "Priority",
      dataIndex: "priority",
      key: "priority",
      width: 80,
      render: (p: string) => <Tag color={priorityColor(p)}>{p}</Tag>,
    },
    {
      title: "Status",
      dataIndex: "status",
      key: "status",
      width: 130,
      render: (s: string) => <Tag color={statusColor(s)}>{s}</Tag>,
    },
    {
      title: "Created",
      dataIndex: "createTime",
      key: "createTime",
      width: 160,
      render: formatTimestamp,
    },
  ];

  return (
    <div>
      <Row gutter={16} style={{ marginBottom: 24 }}>
        <Col span={6}>
          <Card data-testid={tid.dashboard.statComponents}>
            <Statistic title="Components" value={components.length} prefix={<AppstoreOutlined />} />
          </Card>
        </Col>
        <Col span={6}>
          <Card data-testid={tid.dashboard.statOpenIssues}>
            <Statistic title="Open Issues" value={openIssues.length} prefix={<BugOutlined />} valueStyle={{ color: "#1677ff" }} />
          </Card>
        </Col>
        <Col span={6}>
          <Card data-testid={tid.dashboard.statClosed}>
            <Statistic title="Closed" value={closedIssues.length} prefix={<CheckCircleOutlined />} valueStyle={{ color: "#52c41a" }} />
          </Card>
        </Col>
        <Col span={6}>
          <Card data-testid={tid.dashboard.statP0Open}>
            <Statistic title="P0 Open" value={p0Open.length} prefix={<ClockCircleOutlined />} valueStyle={{ color: "#ff4d4f" }} />
          </Card>
        </Col>
      </Row>

      <Card title="Recent Issues" style={{ marginBottom: 24 }}>
        <Table<Issue>
          data-testid={tid.dashboard.recentTable}
          columns={recentColumns}
          dataSource={issues.slice(0, 10)}
          rowKey="issueId"
          pagination={false}
          size="small"
        />
      </Card>
    </div>
  );
}
