import { Table, Card, Tag } from "antd";
import { useListEventsQuery } from "../store/api";
import type { Event } from "../api/types";
import { formatTimestamp } from "../components/formatHelpers";
import { tid } from "../testIds";

export default function EventsPage() {
  const { data, isLoading } = useListEventsQuery();

  const columns = [
    { title: "ID", dataIndex: "eventId", key: "eventId", width: 60 },
    {
      title: "Entity",
      key: "entity",
      width: 140,
      render: (_: unknown, record: Event) => (
        <Tag>{record.entityType} #{record.entityId}</Tag>
      ),
    },
    { title: "Type", dataIndex: "eventType", key: "eventType", width: 160 },
    {
      title: "Payload",
      dataIndex: "payload",
      key: "payload",
      ellipsis: true,
      render: (payload: string) => {
        if (!payload) return "-";
        try {
          const parsed: unknown = JSON.parse(payload);
          return JSON.stringify(parsed);
        } catch {
          return payload;
        }
      },
    },
    { title: "Actor", dataIndex: "actor", key: "actor", width: 180, render: (u: string) => u || "-" },
    { title: "Time", dataIndex: "eventTime", key: "eventTime", width: 160, render: formatTimestamp },
  ];

  return (
    <Card title="Event Log">
      <Table<Event>
        data-testid={tid.events.table}
        columns={columns}
        dataSource={data?.events ?? []}
        rowKey="eventId"
        loading={isLoading}
        pagination={{ pageSize: 30 }}
        size="small"
      />
    </Card>
  );
}
