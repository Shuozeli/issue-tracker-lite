import { useState } from "react";
import { Card, Input, Button, Typography, Space, Avatar, Divider, Row, Col } from "antd";
import {
  UserOutlined, CrownOutlined, BugOutlined, CodeOutlined,
  SafetyOutlined, CustomerServiceOutlined,
} from "@ant-design/icons";
import { useDispatch } from "react-redux";
import { login } from "../store/authSlice";
import { tid } from "../testIds";

const { Title, Text } = Typography;

interface DemoUser {
  email: string;
  name: string;
  role: string;
  icon: React.ReactNode;
  color: string;
}

const demoUsers: DemoUser[] = [
  { email: "alice@payments.dev", name: "Alice Chen", role: "Tech Lead", icon: <CrownOutlined />, color: "#f5222d" },
  { email: "bob@auth.dev", name: "Bob Kumar", role: "Security Engineer", icon: <SafetyOutlined />, color: "#722ed1" },
  { email: "carol@frontend.dev", name: "Carol Zhang", role: "Frontend Engineer", icon: <CodeOutlined />, color: "#13c2c2" },
  { email: "dave@infra.dev", name: "Dave Wilson", role: "SRE / On-Call", icon: <BugOutlined />, color: "#fa8c16" },
  { email: "eve@mobile.dev", name: "Eve Santos", role: "Mobile Engineer", icon: <CodeOutlined />, color: "#52c41a" },
  { email: "frank@support.dev", name: "Frank Lee", role: "Customer Support", icon: <CustomerServiceOutlined />, color: "#2f54eb" },
];

export default function LoginPage() {
  const dispatch = useDispatch();
  const [email, setEmail] = useState("");

  const handleLogin = () => {
    const trimmed = email.trim();
    if (!trimmed) return;
    dispatch(login(trimmed));
  };

  const handleDemoLogin = (user: DemoUser) => {
    dispatch(login(user.email));
  };

  return (
    <div style={{ display: "flex", justifyContent: "center", alignItems: "center", minHeight: "100vh", background: "#141414" }}>
      <div style={{ width: 520 }}>
        <Card>
          <Space direction="vertical" size="large" style={{ width: "100%" }}>
            <div style={{ textAlign: "center" }}>
              <Title level={3} style={{ marginBottom: 4 }}>Issue Tracker</Title>
              <Text type="secondary">Sign in with your email to continue</Text>
            </div>
            <Input
              data-testid={tid.login.email}
              size="large"
              prefix={<UserOutlined />}
              placeholder="you@example.com"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              onPressEnter={handleLogin}
            />
            <Button data-testid={tid.login.submit} type="primary" size="large" block onClick={handleLogin} disabled={!email.trim()}>
              Sign In
            </Button>
          </Space>
        </Card>

        <Divider plain style={{ color: "rgba(255,255,255,0.45)", borderColor: "rgba(255,255,255,0.12)" }}>
          or sign in as a demo user
        </Divider>

        <Row gutter={[12, 12]}>
          {demoUsers.map((user) => (
            <Col span={8} key={user.email}>
              <Card
                hoverable
                size="small"
                style={{ textAlign: "center", cursor: "pointer" }}
                styles={{ body: { padding: "16px 8px" } }}
                onClick={() => handleDemoLogin(user)}
              >
                <Avatar size={40} icon={user.icon} style={{ backgroundColor: user.color, marginBottom: 8 }} />
                <div>
                  <Text strong style={{ fontSize: 13, display: "block" }}>{user.name}</Text>
                  <Text type="secondary" style={{ fontSize: 11 }}>{user.role}</Text>
                </div>
              </Card>
            </Col>
          ))}
        </Row>
      </div>
    </div>
  );
}
