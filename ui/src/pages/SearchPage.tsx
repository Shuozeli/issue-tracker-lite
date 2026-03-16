import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Input, Table, Tag, Card, Typography, Space } from "antd";
import { SearchOutlined } from "@ant-design/icons";
import { useSearchIssuesQuery } from "../store/api";
import type { Issue } from "../api/types";
import { priorityColor, statusColor, typeColor, formatTimestamp } from "../components/formatHelpers";
import { tid } from "../testIds";

const { Text } = Typography;

const EXAMPLE_QUERIES = [
  "status:open",
  "priority:P0",
  "type:BUG",
  "assignee:alice@example.com",
  "status:open priority:P0 type:BUG",
  "-type:TASK",
];

export default function SearchPage() {
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [activeQuery, setActiveQuery] = useState("");
  const { data, isLoading, isFetching } = useSearchIssuesQuery(activeQuery, {
    skip: !activeQuery,
  });

  const handleSearch = (value: string) => {
    setActiveQuery(value);
  };

  const columns = [
    { title: "ID", dataIndex: "issueId", key: "issueId", width: 60 },
    {
      title: "Title",
      dataIndex: "title",
      key: "title",
      ellipsis: true,
      render: (title: string, record: Issue) => (
        <a onClick={() => navigate(`/issues/${record.issueId}`)}>{title}</a>
      ),
    },
    { title: "Type", dataIndex: "type", key: "type", width: 140, render: (t: string) => <Tag color={typeColor(t)}>{t}</Tag> },
    { title: "Priority", dataIndex: "priority", key: "priority", width: 80, render: (p: string) => <Tag color={priorityColor(p)}>{p}</Tag> },
    { title: "Status", dataIndex: "status", key: "status", width: 130, render: (s: string) => <Tag color={statusColor(s)}>{s}</Tag> },
    { title: "Assignee", dataIndex: "assignee", key: "assignee", width: 180, render: (a: string) => a || "-" },
    { title: "Created", dataIndex: "createTime", key: "createTime", width: 160, render: formatTimestamp },
  ];

  return (
    <div>
      <Card style={{ marginBottom: 16 }}>
        <Input.Search
          data-testid={tid.search.input}
          placeholder="Search issues (e.g. status:open priority:P0 type:BUG)"
          allowClear
          enterButton={<><SearchOutlined /> Search</>}
          size="large"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onSearch={handleSearch}
        />
        <div style={{ marginTop: 8 }}>
          <Text type="secondary">Examples: </Text>
          <Space wrap>
            {EXAMPLE_QUERIES.map((eq) => (
              <Tag
                key={eq}
                style={{ cursor: "pointer" }}
                onClick={() => { setQuery(eq); setActiveQuery(eq); }}
              >
                {eq}
              </Tag>
            ))}
          </Space>
        </div>
      </Card>

      {activeQuery && (
        <Card title={`Results${data ? ` (${data.totalCount})` : ""}`}>
          <Table<Issue>
            data-testid={tid.search.resultsTable}
            columns={columns}
            dataSource={data?.issues ?? []}
            rowKey="issueId"
            loading={isLoading || isFetching}
            pagination={{ pageSize: 20 }}
            size="small"
          />
        </Card>
      )}
    </div>
  );
}
