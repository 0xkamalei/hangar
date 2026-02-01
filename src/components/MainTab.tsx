import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Play, Square, Copy, FileEdit, Sparkles, History, RotateCcw, Trash2 } from "lucide-react";

interface AiPatchResult {
  description: string;
  operations: unknown[];
}

interface ConfigVersion {
  id: string;
  timestamp: number;
  description: string;
  file_path: string;
}

export default function MainTab() {
  const { t } = useTranslation();
  const [isRunning, setIsRunning] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [aiPrompt, setAiPrompt] = useState("");
  const [aiLoading, setAiLoading] = useState(false);
  const [aiResult, setAiResult] = useState<AiPatchResult | null>(null);
  const [serverMessage, setServerMessage] = useState("");
  const [versions, setVersions] = useState<ConfigVersion[]>([]);
  const port = 8080;

  useEffect(() => {
    checkServerStatus();
    loadVersions();
  }, []);

  async function checkServerStatus() {
    try {
      const running = await invoke<boolean>("get_server_status");
      setIsRunning(running);
    } catch (error) {
      console.error("Failed to get server status:", error);
    }
  }

  async function loadVersions() {
    try {
      const vers = await invoke<ConfigVersion[]>("list_versions");
      setVersions(vers);
    } catch (error) {
      console.error("Failed to load versions:", error);
    }
  }

  async function startServer() {
    setIsLoading(true);
    try {
      const result = await invoke<string>("start_proxy_server");
      setServerMessage(result);
      setIsRunning(true);
    } catch (error) {
      setServerMessage(`Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function stopServer() {
    setIsLoading(true);
    try {
      const result = await invoke<string>("stop_proxy_server");
      setServerMessage(result);
      setIsRunning(false);
    } catch (error) {
      setServerMessage(`Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  function copySubscriptionUrl() {
    const url = `http://127.0.0.1:${port}/config`;
    navigator.clipboard.writeText(url);
    setServerMessage("✅ 链接已复制到剪贴板");
    setTimeout(() => setServerMessage(""), 2000);
  }

  async function handleAiGenerate() {
    if (!aiPrompt.trim()) return;
    setAiLoading(true);
    setAiResult(null);
    try {
      const result = await invoke<AiPatchResult>("generate_ai_patch", { prompt: aiPrompt });
      setAiResult(result);
      setServerMessage(`✅ AI 生成成功: ${result.description}`);
    } catch (error) {
      setServerMessage(`❌ AI 生成失败: ${error}`);
    } finally {
      setAiLoading(false);
    }
  }

  async function handleRollback(id: string) {
    try {
      await invoke<string>("rollback_version", { id });
      setServerMessage("✅ 已回退到指定版本");
      loadVersions();
    } catch (error) {
      setServerMessage(`❌ 回退失败: ${error}`);
    }
  }

  async function handleDeleteVersion(id: string) {
    if (!confirm("确定要删除这个版本吗？")) return;
    try {
      await invoke<string>("delete_version", { id });
      loadVersions();
    } catch (error) {
      setServerMessage(`❌ 删除失败: ${error}`);
    }
  }

  function formatTimestamp(ts: number) {
    return new Date(ts * 1000).toLocaleString();
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="text-center">
        <h1 className="text-3xl font-bold">{t("main.title")}</h1>
        <p className="text-muted-foreground mt-1">{t("main.subtitle")}</p>
      </div>

      {/* AI 指令输入区 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5" />
            AI 配置助手
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Textarea
            placeholder={t("main.ai_input_placeholder")}
            value={aiPrompt}
            onChange={(e) => setAiPrompt(e.target.value)}
            className="min-h-[100px] resize-none"
          />
          <Button onClick={handleAiGenerate} disabled={!aiPrompt.trim() || aiLoading}>
            <Sparkles className="h-4 w-4 mr-2" />
            {aiLoading ? "生成中..." : t("main.generate")}
          </Button>

          {/* AI Result Preview */}
          {aiResult && (
            <div className="mt-4 p-4 bg-muted rounded-lg">
              <h4 className="font-medium mb-2">AI 修改预览</h4>
              <p className="text-sm text-muted-foreground mb-2">{aiResult.description}</p>
              <pre className="text-xs bg-background p-2 rounded overflow-auto max-h-40">
                {JSON.stringify(aiResult.operations, null, 2)}
              </pre>
              <div className="flex gap-2 mt-3">
                <Button size="sm">确认应用</Button>
                <Button size="sm" variant="outline" onClick={() => setAiResult(null)}>
                  取消
                </Button>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* 服务器控制 */}
      <Card>
        <CardHeader>
          <CardTitle>{t("main.server_control")}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* 状态指示器 */}
          <div className="flex items-center gap-3">
            <div
              className={`w-3 h-3 rounded-full ${
                isRunning ? "bg-green-500" : "bg-red-500"
              }`}
            />
            <span className="font-medium">
              {isRunning ? t("main.server_running") : t("main.server_stopped")}
            </span>
          </div>

          {/* 控制按钮 */}
          <div className="flex gap-3">
            <Button
              onClick={startServer}
              disabled={isLoading || isRunning}
              variant={isRunning ? "secondary" : "default"}
            >
              <Play className="h-4 w-4 mr-2" />
              {t("main.server_start")}
            </Button>
            <Button
              onClick={stopServer}
              disabled={isLoading || !isRunning}
              variant="destructive"
            >
              <Square className="h-4 w-4 mr-2" />
              {t("main.server_stop")}
            </Button>
          </div>

          {/* 订阅链接 */}
          {isRunning && (
            <div className="flex items-center gap-3 p-3 bg-muted rounded-lg">
              <span className="text-sm text-muted-foreground">
                {t("main.subscription_url")}:
              </span>
              <code className="text-sm font-mono flex-1">
                http://127.0.0.1:{port}/config
              </code>
              <Button size="sm" variant="outline" onClick={copySubscriptionUrl}>
                <Copy className="h-4 w-4" />
              </Button>
            </div>
          )}

          {/* 状态消息 */}
          {serverMessage && (
            <div
              className={`p-3 rounded-lg text-sm ${
                serverMessage.includes("Error") || serverMessage.includes("错误") || serverMessage.includes("失败")
                  ? "bg-destructive/10 text-destructive"
                  : "bg-green-500/10 text-green-700"
              }`}
            >
              {serverMessage}
            </div>
          )}
        </CardContent>
      </Card>

      {/* 版本历史 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <History className="h-5 w-5" />
            {t("main.version_history")}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {versions.length === 0 ? (
            <p className="text-sm text-muted-foreground">暂无版本历史</p>
          ) : (
            <div className="space-y-2 max-h-60 overflow-auto">
              {versions.map((v) => (
                <div
                  key={v.id}
                  className="flex items-center justify-between p-2 bg-muted rounded"
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium truncate">{v.description || "未命名"}</p>
                    <p className="text-xs text-muted-foreground">
                      {formatTimestamp(v.timestamp)}
                    </p>
                  </div>
                  <div className="flex gap-1">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleRollback(v.id)}
                    >
                      <RotateCcw className="h-4 w-4" />
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleDeleteVersion(v.id)}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* 快捷操作 */}
      <div className="flex gap-3">
        <Button variant="outline">
          <FileEdit className="h-4 w-4 mr-2" />
          {t("main.open_editor")}
        </Button>
      </div>
    </div>
  );
}
