import { useState } from "react";
import { Table, Card, Button, Modal, Form, Input } from "antd";
import { PlusOutlined } from "@ant-design/icons";
import { useListHotlistsQuery, useCreateHotlistMutation } from "../store/api";
import type { Hotlist, CreateHotlistRequest } from "../api/types";
import { formatTimestamp } from "../components/formatHelpers";
import { tid } from "../testIds";

export default function HotlistsPage() {
  const { data, isLoading } = useListHotlistsQuery();
  const [createHotlist] = useCreateHotlistMutation();
  const [modalOpen, setModalOpen] = useState(false);
  const [form] = Form.useForm<CreateHotlistRequest>();

  const columns = [
    { title: "ID", dataIndex: "hotlistId", key: "hotlistId", width: 60 },
    { title: "Name", dataIndex: "name", key: "name" },
    { title: "Description", dataIndex: "description", key: "description", ellipsis: true },
    { title: "Owner", dataIndex: "owner", key: "owner", width: 180 },
    { title: "Issues", dataIndex: "issueCount", key: "issueCount", width: 80 },
    { title: "Created", dataIndex: "createTime", key: "createTime", width: 160, render: formatTimestamp },
  ];

  const handleCreate = async () => {
    const values = await form.validateFields();
    await createHotlist(values);
    setModalOpen(false);
    form.resetFields();
  };

  return (
    <Card
      title="Hotlists"
      extra={
        <Button data-testid={tid.hotlists.createBtn} type="primary" icon={<PlusOutlined />} onClick={() => setModalOpen(true)}>
          New Hotlist
        </Button>
      }
    >
      <Table<Hotlist>
        data-testid={tid.hotlists.table}
        columns={columns}
        dataSource={data?.hotlists ?? []}
        rowKey="hotlistId"
        loading={isLoading}
        pagination={false}
        size="small"
      />
      <Modal title="Create Hotlist" open={modalOpen} onOk={handleCreate} onCancel={() => setModalOpen(false)}>
        <Form form={form} layout="vertical">
          <Form.Item name="name" label="Name" rules={[{ required: true }]}>
            <Input data-testid={tid.hotlists.inputName} />
          </Form.Item>
          <Form.Item name="description" label="Description">
            <Input.TextArea data-testid={tid.hotlists.inputDescription} rows={2} />
          </Form.Item>
          <Form.Item name="owner" label="Owner">
            <Input data-testid={tid.hotlists.inputOwner} placeholder="email@example.com" />
          </Form.Item>
        </Form>
      </Modal>
    </Card>
  );
}
