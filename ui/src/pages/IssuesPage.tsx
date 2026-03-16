import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Table, Tag, Button, Modal, Form, Input, Select, Card, Space } from "antd";
import { PlusOutlined } from "@ant-design/icons";
import { useSelector } from "react-redux";
import type { RootState } from "../store";
import { useListIssuesQuery, useCreateIssueMutation, useListComponentsQuery } from "../store/api";
import type { Issue, CreateIssueRequest, IssueType, Priority, Severity } from "../api/types";
import { priorityColor, statusColor, typeColor, formatTimestamp } from "../components/formatHelpers";
import { tid } from "../testIds";

const issueTypes: IssueType[] = ["BUG", "FEATURE_REQUEST", "TASK", "VULNERABILITY", "CUSTOMER_ISSUE", "INTERNAL_CLEANUP"];
const priorities: Priority[] = ["P0", "P1", "P2", "P3", "P4"];
const severities: Severity[] = ["S0", "S1", "S2", "S3", "S4"];

export default function IssuesPage() {
  const navigate = useNavigate();
  const userId = useSelector((state: RootState) => state.auth.userId) ?? "";
  const { data, isLoading } = useListIssuesQuery();
  const { data: compData } = useListComponentsQuery();
  const [createIssue] = useCreateIssueMutation();
  const [modalOpen, setModalOpen] = useState(false);
  const [form] = Form.useForm<CreateIssueRequest>();

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
    {
      title: "Type",
      dataIndex: "type",
      key: "type",
      width: 140,
      render: (t: string) => <Tag color={typeColor(t)}>{t}</Tag>,
    },
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
      title: "Assignee",
      dataIndex: "assignee",
      key: "assignee",
      width: 180,
      render: (a: string) => a || "-",
    },
    {
      title: "Created",
      dataIndex: "createTime",
      key: "createTime",
      width: 160,
      render: formatTimestamp,
    },
  ];

  const handleCreate = async () => {
    const values = await form.validateFields();
    await createIssue(values);
    setModalOpen(false);
    form.resetFields();
  };

  return (
    <div>
      <Card
        title="Issues"
        extra={
          <Button data-testid={tid.issues.createBtn} type="primary" icon={<PlusOutlined />} onClick={() => setModalOpen(true)}>
            New Issue
          </Button>
        }
      >
        <Table<Issue>
          data-testid={tid.issues.table}
          columns={columns}
          dataSource={data?.issues ?? []}
          rowKey="issueId"
          loading={isLoading}
          pagination={{ pageSize: 20 }}
          size="small"
        />
      </Card>

      <Modal
        title="Create Issue"
        open={modalOpen}
        onOk={handleCreate}
        onCancel={() => setModalOpen(false)}
        width={600}
      >
        <Form form={form} layout="vertical">
          <Form.Item name="componentId" label="Component" rules={[{ required: true }]}>
            <Select data-testid={tid.issues.selectComponent} placeholder="Select component" showSearch optionFilterProp="label"
              options={(compData?.components ?? []).map((c) => ({
                value: c.componentId, label: c.name,
              }))}
            />
          </Form.Item>
          <Form.Item name="title" label="Title" rules={[{ required: true }]}>
            <Input data-testid={tid.issues.inputTitle} />
          </Form.Item>
          <Form.Item name="description" label="Description">
            <Input.TextArea data-testid={tid.issues.inputDescription} rows={3} />
          </Form.Item>
          <Space size="middle">
            <Form.Item name="type" label="Type" initialValue="BUG">
              <Select data-testid={tid.issues.selectType} style={{ width: 160 }}>
                {issueTypes.map((t) => <Select.Option key={t} value={t}>{t}</Select.Option>)}
              </Select>
            </Form.Item>
            <Form.Item name="priority" label="Priority" initialValue="P2">
              <Select data-testid={tid.issues.selectPriority} style={{ width: 100 }}>
                {priorities.map((p) => <Select.Option key={p} value={p}>{p}</Select.Option>)}
              </Select>
            </Form.Item>
            <Form.Item name="severity" label="Severity">
              <Select data-testid={tid.issues.selectSeverity} style={{ width: 100 }} allowClear placeholder="None">
                {severities.map((s) => <Select.Option key={s} value={s}>{s}</Select.Option>)}
              </Select>
            </Form.Item>
          </Space>
          <Form.Item name="assignee" label="Assignee">
            <Input data-testid={tid.issues.inputAssignee} placeholder="email@example.com" />
          </Form.Item>
          <Form.Item name="reporter" label="Reporter" initialValue={userId}>
            <Input data-testid={tid.issues.inputReporter} placeholder="email@example.com" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
