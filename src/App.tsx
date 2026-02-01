import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface Subscription {
  name: string;
  url: string;
  enabled: boolean;
}

function App() {
  const [serverStatus, setServerStatus] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isRunning, setIsRunning] = useState(false);
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editIndex, setEditIndex] = useState<number | null>(null);
  const [formData, setFormData] = useState({ name: "", url: "", enabled: true });
  const [statusTimeout, setStatusTimeout] = useState<number | null>(null);

  // 加载订阅列表
  async function loadSubscriptions() {
    try {
      const subs = await invoke<Subscription[]>("get_subscriptions");
      setSubscriptions(subs);
    } catch (error) {
      console.error("加载订阅失败:", error);
    }
  }

  // 检查服务器状态
  async function checkServerStatus() {
    try {
      const running = await invoke<boolean>("get_server_status");
      setIsRunning(running);
    } catch (error) {
      console.error("获取服务器状态失败:", error);
    }
  }

  // 组件加载时检查服务器状态和加载订阅
  useEffect(() => {
    checkServerStatus();
    loadSubscriptions();
  }, []);

  // 显示状态消息并自动隐藏
  function showStatus(message: string) {
    setServerStatus(message);
    
    // 清除之前的定时器
    if (statusTimeout) {
      clearTimeout(statusTimeout);
    }
    
    // 3秒后自动隐藏
    const timeout = window.setTimeout(() => {
      setServerStatus("");
    }, 3000);
    
    setStatusTimeout(timeout);
  }

  async function startServer() {
    console.log("🔍 开始启动服务器...");
    setIsLoading(true);
    showStatus("正在启动服务器...");
    
    try {
      console.log("✓ 调用 start_proxy_server 命令");
      const result = await invoke<string>("start_proxy_server");
      console.log(`✓ 服务器启动成功: ${result}`);
      showStatus(result);
      setIsRunning(true);
    } catch (error) {
      console.error("❌ 服务器启动失败:", error);
      showStatus(`错误: ${error}`);
      setIsRunning(false);
    } finally {
      setIsLoading(false);
      console.log("✓ startServer 函数执行完毕");
    }
  }

  async function stopServer() {
    setIsLoading(true);
    showStatus("正在停止服务器...");
    
    try {
      const result = await invoke<string>("stop_proxy_server");
      showStatus(result);
      setIsRunning(false);
    } catch (error) {
      showStatus(`错误: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleAddSubscription() {
    if (!formData.name || !formData.url) {
      showStatus("错误: 请填写订阅名称和URL");
      return;
    }

    try {
      const result = await invoke<string>("add_subscription", {
        name: formData.name,
        url: formData.url,
      });
      showStatus(result);
      setFormData({ name: "", url: "", enabled: true });
      setShowAddForm(false);
      await loadSubscriptions();
    } catch (error) {
      showStatus(`错误: ${error}`);
    }
  }

  async function handleUpdateSubscription() {
    if (editIndex === null) return;
    
    try {
      const result = await invoke<string>("update_subscription", {
        index: editIndex,
        name: formData.name,
        url: formData.url,
        enabled: formData.enabled,
      });
      showStatus(result);
      setEditIndex(null);
      setFormData({ name: "", url: "", enabled: true });
      await loadSubscriptions();
    } catch (error) {
      showStatus(`错误: ${error}`);
    }
  }

  async function handleDeleteSubscription(index: number) {
    console.log(`🔍 准备删除订阅，index: ${index}`);
    
    if (!confirm("确定要删除这个订阅吗？")) {
      console.log("❌ 用户取消删除");
      return;
    }
    
    console.log(`✓ 用户确认删除，调用 delete_subscription`);
    
    try {
      const result = await invoke<string>("delete_subscription", { index });
      console.log(`✓ 删除成功: ${result}`);
      showStatus(result);
      await loadSubscriptions();
    } catch (error) {
      console.error(`❌ 删除失败:`, error);
      showStatus(`错误: ${error}`);
    }
  }

  async function handleToggleEnabled(index: number) {
    const sub = subscriptions[index];
    try {
      await invoke<string>("update_subscription", {
        index,
        name: sub.name,
        url: sub.url,
        enabled: !sub.enabled,
      });
      await loadSubscriptions();
    } catch (error) {
      showStatus(`错误: ${error}`);
    }
  }

  function startEdit(index: number) {
    const sub = subscriptions[index];
    setEditIndex(index);
    setFormData({ name: sub.name, url: sub.url, enabled: sub.enabled });
    setShowAddForm(false);
  }

  function cancelEdit() {
    setEditIndex(null);
    setShowAddForm(false);
    setFormData({ name: "", url: "", enabled: true });
  }

  return (
    <>
      {/* macOS 原生风格拖拽区域 */}
      <div className="titlebar" />
      <main className="container">
      <h1>代理订阅管理器</h1>
      <p>Proxy Subscription Manager</p>

      <div style={{ 
        marginTop: "2rem",
        display: "flex",
        alignItems: "center",
        gap: "1rem",
        justifyContent: "center"
      }}>
        <div style={{
          display: "inline-block",
          width: "12px",
          height: "12px",
          borderRadius: "50%",
          backgroundColor: isRunning ? "#22c55e" : "#ef4444",
          marginRight: "0.5rem"
        }} />
        <span style={{ fontWeight: "500" }}>
          {isRunning ? "运行中" : "已停止"}
        </span>
      </div>

      <div style={{ 
        marginTop: "1.5rem",
        display: "flex",
        gap: "1rem",
        justifyContent: "center"
      }}>
        <button 
          onClick={startServer} 
          disabled={isLoading || isRunning}
          style={{
            padding: "12px 24px",
            fontSize: "16px",
            cursor: (isLoading || isRunning) ? "not-allowed" : "pointer",
            backgroundColor: (isLoading || isRunning) ? "#ccc" : "#0070f3",
            color: "white",
            border: "none",
            borderRadius: "6px",
            fontWeight: "500"
          }}
        >
          {isLoading ? "处理中..." : "启动服务器"}
        </button>

        <button 
          onClick={stopServer} 
          disabled={isLoading || !isRunning}
          style={{
            padding: "12px 24px",
            fontSize: "16px",
            cursor: (isLoading || !isRunning) ? "not-allowed" : "pointer",
            backgroundColor: (isLoading || !isRunning) ? "#ccc" : "#dc2626",
            color: "white",
            border: "none",
            borderRadius: "6px",
            fontWeight: "500"
          }}
        >
          停止服务器
        </button>
      </div>

      {serverStatus && (
        <div style={{
          marginTop: "2rem",
          padding: "1rem",
          backgroundColor: serverStatus.includes("错误") ? "#fee2e2" : "#d1fae5",
          borderRadius: "6px",
          whiteSpace: "pre-wrap",
          border: serverStatus.includes("错误") ? "1px solid #fca5a5" : "1px solid #86efac",
          color: serverStatus.includes("错误") ? "#991b1b" : "#065f46",
          fontWeight: "500"
        }}>
          {serverStatus}
        </div>
      )}

      {/* 订阅管理 */}
      <div style={{ marginTop: "3rem", width: "100%", maxWidth: "800px" }}>
        <div style={{ 
          display: "flex", 
          justifyContent: "space-between", 
          alignItems: "center",
          marginBottom: "1rem"
        }}>
          <h3 style={{ margin: 0 }}>订阅管理</h3>
          <button
            onClick={() => {
              setShowAddForm(true);
              setEditIndex(null);
              setFormData({ name: "", url: "", enabled: true });
            }}
            style={{
              padding: "8px 16px",
              fontSize: "14px",
              backgroundColor: "#0070f3",
              color: "white",
              border: "none",
              borderRadius: "6px",
              cursor: "pointer",
              fontWeight: "500"
            }}
          >
            + 添加订阅
          </button>
        </div>

        {/* 添加/编辑表单 */}
        {(showAddForm || editIndex !== null) && (
          <div className="form-container">
            <h4 style={{ marginTop: 0 }}>
              {editIndex !== null ? "编辑订阅" : "添加新订阅"}
            </h4>
            <div style={{ marginBottom: "1rem" }}>
              <label className="form-label">
                订阅名称:
              </label>
              <input
                type="text"
                className="form-input"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
              />
            </div>
            <div style={{ marginBottom: "1rem" }}>
              <label className="form-label">
                订阅链接:
              </label>
              <input
                type="text"
                className="form-input"
                value={formData.url}
                onChange={(e) => setFormData({ ...formData, url: e.target.value })}
              />
            </div>
            {editIndex !== null && (
              <div style={{ marginBottom: "1rem" }}>
                <label style={{ display: "flex", alignItems: "center", cursor: "pointer" }}>
                  <input
                    type="checkbox"
                    checked={formData.enabled}
                    onChange={(e) => setFormData({ ...formData, enabled: e.target.checked })}
                    style={{ marginRight: "0.5rem" }}
                  />
                  启用此订阅
                </label>
              </div>
            )}
            <div style={{ display: "flex", gap: "0.5rem" }}>
              <button
                onClick={editIndex !== null ? handleUpdateSubscription : handleAddSubscription}
                style={{
                  padding: "8px 16px",
                  fontSize: "14px",
                  backgroundColor: "#28a745",
                  color: "white",
                  border: "none",
                  borderRadius: "4px",
                  cursor: "pointer",
                  fontWeight: "500"
                }}
              >
                {editIndex !== null ? "保存" : "添加"}
              </button>
              <button
                onClick={cancelEdit}
                style={{
                  padding: "8px 16px",
                  fontSize: "14px",
                  backgroundColor: "#6c757d",
                  color: "white",
                  border: "none",
                  borderRadius: "4px",
                  cursor: "pointer",
                  fontWeight: "500"
                }}
              >
                取消
              </button>
            </div>
          </div>
        )}

        {/* 订阅列表 */}
        <div style={{ display: "flex", flexDirection: "column", gap: "0.75rem" }}>
          {subscriptions.length === 0 ? (
            <div style={{
              padding: "2rem",
              textAlign: "center",
              backgroundColor: "#f8f9fa",
              borderRadius: "8px",
              color: "#6c757d"
            }}>
              暂无订阅，点击"添加订阅"按钮开始
            </div>
          ) : (
            subscriptions.map((sub, index) => (
              <div
                key={index}
                className={`subscription-card ${sub.enabled ? "enabled" : "disabled"}`}
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center"
                }}
              >
                <div style={{ flex: 1 }}>
                  <div style={{ 
                    fontWeight: "600", 
                    fontSize: "16px",
                    opacity: sub.enabled ? 1 : 0.6,
                    marginBottom: "0.25rem"
                  }}>
                    {sub.name}
                    {sub.enabled && (
                      <span style={{
                        marginLeft: "0.5rem",
                        fontSize: "12px",
                        padding: "2px 8px",
                        backgroundColor: "#28a745",
                        color: "white",
                        borderRadius: "12px"
                      }}>
                        启用
                      </span>
                    )}
                  </div>
                  <div style={{ 
                    fontSize: "13px", 
                    color: "#6c757d",
                    wordBreak: "break-all"
                  }}>
                    {sub.url}
                  </div>
                </div>
                <div style={{ display: "flex", gap: "0.5rem", marginLeft: "1rem" }}>
                  <button
                    onClick={() => handleToggleEnabled(index)}
                    style={{
                      padding: "6px 12px",
                      fontSize: "13px",
                      backgroundColor: sub.enabled ? "#ffc107" : "#28a745",
                      color: "white",
                      border: "none",
                      borderRadius: "4px",
                      cursor: "pointer",
                      whiteSpace: "nowrap"
                    }}
                  >
                    {sub.enabled ? "禁用" : "启用"}
                  </button>
                  <button
                    onClick={() => startEdit(index)}
                    style={{
                      padding: "6px 12px",
                      fontSize: "13px",
                      backgroundColor: "#0070f3",
                      color: "white",
                      border: "none",
                      borderRadius: "4px",
                      cursor: "pointer"
                    }}
                  >
                    编辑
                  </button>
                  <button
                    onClick={() => handleDeleteSubscription(index)}
                    style={{
                      padding: "6px 12px",
                      fontSize: "13px",
                      backgroundColor: "#dc2626",
                      color: "white",
                      border: "none",
                      borderRadius: "4px",
                      cursor: "pointer"
                    }}
                  >
                    删除
                  </button>
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </main>
    </>
  );
}

export default App;
