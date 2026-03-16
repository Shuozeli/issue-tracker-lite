import { useState, useEffect, useCallback } from "react";
import { Routes, Route, useNavigate, useLocation } from "react-router-dom";
import { Layout, Menu, Button, Typography, Space, Tooltip } from "antd";
import {
  DashboardOutlined,
  BugOutlined,
  AppstoreOutlined,
  UnorderedListOutlined,
  SearchOutlined,
  HistoryOutlined,
  LogoutOutlined,
  UserOutlined,
  CodeOutlined,
} from "@ant-design/icons";
import { useSelector, useDispatch } from "react-redux";
import type { RootState } from "./store";
import { logout } from "./store/authSlice";
import DashboardPage from "./pages/DashboardPage";
import IssuesPage from "./pages/IssuesPage";
import IssueDetailPage from "./pages/IssueDetailPage";
import ComponentsPage from "./pages/ComponentsPage";
import HotlistsPage from "./pages/HotlistsPage";
import SearchPage from "./pages/SearchPage";
import EventsPage from "./pages/EventsPage";
import LoginPage from "./pages/LoginPage";
import { DemoConsole } from "./components/DemoConsole";
import { tid } from "./testIds";

const { Sider, Content, Header } = Layout;
const { Text } = Typography;

const menuItems = [
  { key: "/", icon: <DashboardOutlined />, label: "Dashboard" },
  { key: "/issues", icon: <BugOutlined />, label: "Issues" },
  { key: "/components", icon: <AppstoreOutlined />, label: "Components" },
  { key: "/hotlists", icon: <UnorderedListOutlined />, label: "Hotlists" },
  { key: "/search", icon: <SearchOutlined />, label: "Search" },
  { key: "/events", icon: <HistoryOutlined />, label: "Events" },
];

export default function App() {
  const navigate = useNavigate();
  const location = useLocation();
  const dispatch = useDispatch();
  const userId = useSelector((state: RootState) => state.auth.userId);
  const [consoleVisible, setConsoleVisible] = useState(false);

  const toggleConsole = useCallback(() => {
    setConsoleVisible((v) => !v);
  }, []);

  // Keyboard shortcut: Ctrl+` to toggle console
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "`") {
        e.preventDefault();
        toggleConsole();
      }
      if (e.key === "Escape" && consoleVisible) {
        const active = document.activeElement?.tagName;
        if (active !== "INPUT" && active !== "TEXTAREA") {
          setConsoleVisible(false);
        }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleConsole, consoleVisible]);

  if (!userId) {
    return (
      <div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
        <div style={{ flex: 1, overflow: "auto", position: "relative" }}>
          <LoginPage />
          <Tooltip title="Console (Ctrl+`)">
            <Button
              icon={<CodeOutlined />}
              type={consoleVisible ? "primary" : "default"}
              onClick={toggleConsole}
              size="small"
              style={{ position: "fixed", top: 12, right: 12, zIndex: 1000 }}
            />
          </Tooltip>
        </div>
        {consoleVisible && (
          <div style={{ height: 200, borderTop: "1px solid #303030", flexShrink: 0 }}>
            <DemoConsole />
          </div>
        )}
      </div>
    );
  }

  const selectedKey = menuItems.find((item) =>
    item.key === "/" ? location.pathname === "/" : location.pathname.startsWith(item.key),
  )?.key ?? "/";

  return (
    <Layout style={{ height: "100vh" }}>
      <Sider collapsible width={200}>
        <div
          style={{
            height: 48,
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: "#fff",
            fontWeight: 700,
            fontSize: 16,
          }}
        >
          IssueTracker
        </div>
        <Menu
          theme="dark"
          mode="inline"
          selectedKeys={[selectedKey]}
          items={menuItems}
          onClick={({ key }) => navigate(key)}
        />
      </Sider>
      <Layout>
        <Header
          style={{
            background: "#141414",
            padding: "0 24px",
            borderBottom: "1px solid #303030",
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
          }}
        >
          <Text strong style={{ fontSize: 18 }}>Issue Tracker</Text>
          <Space>
            <UserOutlined />
            <Text data-testid={tid.header.userId}>{userId}</Text>
            <Button
              data-testid={tid.header.signOut}
              type="text"
              icon={<LogoutOutlined />}
              onClick={() => dispatch(logout())}
              size="small"
            >
              Sign Out
            </Button>
            <Tooltip title="Console (Ctrl+`)">
              <Button
                icon={<CodeOutlined />}
                type={consoleVisible ? "primary" : "default"}
                onClick={toggleConsole}
                size="small"
              />
            </Tooltip>
          </Space>
        </Header>

        <div style={{ display: "flex", flexDirection: "column", flex: 1, overflow: "hidden" }}>
          <Content style={{ flex: 1, overflow: "auto", padding: 24 }}>
            <Routes>
              <Route path="/" element={<DashboardPage />} />
              <Route path="/issues" element={<IssuesPage />} />
              <Route path="/issues/:id" element={<IssueDetailPage />} />
              <Route path="/components" element={<ComponentsPage />} />
              <Route path="/hotlists" element={<HotlistsPage />} />
              <Route path="/search" element={<SearchPage />} />
              <Route path="/events" element={<EventsPage />} />
            </Routes>
          </Content>

          {consoleVisible && (
            <div style={{ height: 200, borderTop: "1px solid #303030", flexShrink: 0 }}>
              <DemoConsole />
            </div>
          )}
        </div>
      </Layout>
    </Layout>
  );
}
