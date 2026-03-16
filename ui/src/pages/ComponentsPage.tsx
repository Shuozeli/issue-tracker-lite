import { useState } from "react";
import { Table, Card, Button, Modal, Form, Input, Tag } from "antd";
import { PlusOutlined } from "@ant-design/icons";
import {
  useListComponentsQuery,
  useCreateComponentMutation,
  useDeleteComponentMutation,
} from "../store/api";
import type { Component, CreateComponentRequest } from "../api/types";
import { formatTimestamp } from "../components/formatHelpers";
import { tid } from "../testIds";

export default function ComponentsPage() {
  const { data, isLoading } = useListComponentsQuery();
  const [createComponent] = useCreateComponentMutation();
  const [deleteComponent] = useDeleteComponentMutation();
  const [modalOpen, setModalOpen] = useState(false);
  const [form] = Form.useForm<CreateComponentRequest>();

  const columns = [
    { title: "ID", dataIndex: "componentId", key: "componentId", width: 60 },
    { title: "Name", dataIndex: "name", key: "name" },
    {
      title: "Description",
      dataIndex: "description",
      key: "description",
      ellipsis: true,
    },
    {
      title: "Children",
      dataIndex: "childCount",
      key: "childCount",
      width: 80,
      render: (count: number) => <Tag>{count}</Tag>,
    },
    {
      title: "Created",
      dataIndex: "createTime",
      key: "createTime",
      width: 160,
      render: formatTimestamp,
    },
    {
      title: "Actions",
      key: "actions",
      width: 100,
      render: (_: unknown, record: Component) => (
        <Button danger size="small" onClick={() => deleteComponent(record.componentId)}>
          Delete
        </Button>
      ),
    },
  ];

  const handleCreate = async () => {
    const values = await form.validateFields();
    await createComponent(values);
    setModalOpen(false);
    form.resetFields();
  };

  return (
    <div>
      <Card
        title="Components"
        extra={
          <Button data-testid={tid.components.createBtn} type="primary" icon={<PlusOutlined />} onClick={() => setModalOpen(true)}>
            New Component
          </Button>
        }
      >
        <Table<Component>
          data-testid={tid.components.table}
          columns={columns}
          dataSource={data?.components ?? []}
          rowKey="componentId"
          loading={isLoading}
          pagination={false}
          size="small"
        />
      </Card>

      <Modal
        title="Create Component"
        open={modalOpen}
        onOk={handleCreate}
        onCancel={() => setModalOpen(false)}
      >
        <Form form={form} layout="vertical">
          <Form.Item name="name" label="Name" rules={[{ required: true }]}>
            <Input data-testid={tid.components.inputName} />
          </Form.Item>
          <Form.Item name="description" label="Description">
            <Input.TextArea data-testid={tid.components.inputDescription} rows={2} />
          </Form.Item>
          <Form.Item name="parentId" label="Parent ID">
            <Input data-testid={tid.components.inputParentId} type="number" placeholder="Optional" />
          </Form.Item>
        </Form>
      </Modal>
    </div>
  );
}
