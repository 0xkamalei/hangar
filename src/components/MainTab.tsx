import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Sparkles, Zap, StopCircle, FileEdit, Copy, FileText, Clock, History, PlusCircle, Mic, Paperclip, Network } from "lucide-react";

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
  const { t: _t } = useTranslation();
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
    <div className="flex h-full">
      {/* Dashboard Main Content */}
      <div className="flex-1 flex flex-col overflow-y-auto">
        {/* AI Composer Header */}
        <header className="sticky top-0 z-10 bg-background/80 backdrop-blur-md px-8 py-6">
          <div className="max-w-4xl mx-auto">
            <div className="flex flex-col w-full rounded-xl overflow-hidden shadow-2xl shadow-primary/5 border border-border bg-card dark:bg-surface-dark">
              <div className="flex flex-1 items-stretch">
                <div className="flex justify-center items-start pt-[18px] px-4 border-r border-border">
                  <div className="bg-primary/10 text-primary rounded-full size-10 flex items-center justify-center shrink-0">
                    <Sparkles size={20} />
                  </div>
                </div>
                <textarea
                  className="flex w-full min-w-0 flex-1 resize-none overflow-hidden text-foreground bg-transparent focus:outline-0 focus:ring-0 border-0 h-auto placeholder:text-muted-foreground p-4 text-base font-normal leading-normal pt-[22px]"
                  placeholder="How can I help with your configuration today? (e.g., 'Route all Netflix traffic through Japan nodes')"
                  value={aiPrompt}
                  onChange={(e) => setAiPrompt(e.target.value)}
                />
              </div>
              <div className="flex justify-end p-4 pt-0">
                <div className="flex items-center gap-4">
                  <div className="flex items-center gap-1">
                    <button className="p-2 text-muted-foreground hover:text-primary transition-colors">
                      <Mic size={20} />
                    </button>
                    <button className="p-2 text-muted-foreground hover:text-primary transition-colors">
                      <Paperclip size={20} />
                    </button>
                  </div>
                  <Button
                    size="sm"
                    className="gap-2 bg-primary text-white shadow-lg shadow-primary/20"
                    onClick={handleAiGenerate}
                    disabled={!aiPrompt.trim() || aiLoading}
                  >
                    <Zap size={16} />
                    <span>{aiLoading ? "Generating..." : "Generate"}</span>
                  </Button>
                </div>
              </div>
            </div>
          </div>
        </header>

        <div className="px-8 pb-12">
          <div className="max-w-4xl mx-auto space-y-8">
            {/* AI Result Preview */}
            {aiResult && (
              <section className="p-6 rounded-xl border border-primary/20 bg-primary/5 shadow-sm">
                <div className="flex items-center gap-2 mb-4 text-primary">
                  <Sparkles size={20} />
                  <h3 className="font-bold">AI Modification Preview</h3>
                </div>
                <p className="text-sm text-foreground mb-4">{aiResult.description}</p>
                <pre className="text-xs bg-card p-4 rounded-lg border border-border overflow-auto max-h-40 font-mono mb-4">
                  {JSON.stringify(aiResult.operations, null, 2)}
                </pre>
                <div className="flex gap-3">
                  <Button size="sm">Apply Changes</Button>
                  <Button size="sm" variant="outline" onClick={() => setAiResult(null)}>
                    Discard
                  </Button>
                </div>
              </section>
            )}

            {/* Server Control */}
            <section>
              <h2 className="text-foreground text-xl font-bold leading-tight tracking-tight pb-4">Server Control</h2>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div className="md:col-span-2 flex items-stretch justify-between gap-6 rounded-xl border border-border bg-card dark:bg-surface-dark p-6 shadow-sm">
                  <div className="flex flex-col justify-between flex-1">
                    <div className="flex flex-col gap-1">
                      <div className="flex items-center gap-2">
                        <div className={`size-2.5 rounded-full ${isRunning ? "bg-emerald-500 animate-pulse" : "bg-red-500"}`} />
                        <p className="text-foreground text-lg font-bold leading-tight">
                          {isRunning ? "System Active" : "System Stopped"}
                        </p>
                      </div>
                      <p className="text-muted-foreground text-sm font-normal leading-normal mt-1">
                        Core is running on local port <span className="text-primary font-mono font-medium">{port}</span>
                      </p>
                    </div>
                    <div className="flex gap-3 mt-6">
                      {isRunning ? (
                        <Button
                          variant="outline"
                          className="h-10 px-5 text-foreground hover:bg-red-500/10 hover:text-red-500 border-border dark:hover:border-red-500/30 font-bold gap-2"
                          onClick={stopServer}
                          disabled={isLoading}
                        >
                          <StopCircle size={20} />
                          <span>Stop Server</span>
                        </Button>
                      ) : (
                        <Button
                          className="h-10 px-5 bg-primary text-white shadow-lg shadow-primary/20 font-bold gap-2"
                          onClick={startServer}
                          disabled={isLoading}
                        >
                          <Zap size={20} />
                          <span>Start Server</span>
                        </Button>
                      )}
                      <Button
                        variant="outline"
                        className="h-10 px-5 border-border text-foreground hover:bg-muted font-bold gap-2"
                      >
                        <FileEdit size={20} />
                        <span>Open in Editor</span>
                      </Button>
                    </div>
                  </div>
                  <div className="hidden sm:block w-40 bg-primary rounded-lg overflow-hidden border border-border">
                    <div className="w-full h-full flex items-center justify-center opacity-30 bg-gradient-to-br from-primary to-blue-900">
                      <Network className="text-white" size={64} />
                    </div>
                  </div>
                </div>
                <div className="flex flex-col gap-4">
                  <div className="p-4 rounded-xl border border-border bg-card dark:bg-surface-dark flex flex-col">
                    <span className="text-muted-foreground text-xs font-medium uppercase tracking-wider">Active Proxies</span>
                    <span className="text-2xl font-bold text-foreground mt-1">12 / 48</span>
                  </div>
                  <div className="p-4 rounded-xl border border-border bg-card dark:bg-surface-dark flex flex-col">
                    <span className="text-muted-foreground text-xs font-medium uppercase tracking-wider">Traffic Mode</span>
                    <span className="text-xl font-bold text-primary mt-1">Smart Rule</span>
                  </div>
                </div>
              </div>
            </section>

            {/* Subscription URL */}
            <section className="max-w-2xl">
              <div className="flex flex-col gap-3">
                <label className="flex flex-col">
                  <p className="text-foreground text-base font-semibold leading-normal pb-2">Local Subscription URL</p>
                  <div className="flex w-full items-stretch rounded-lg shadow-sm">
                    <input
                      className="flex w-full min-w-0 flex-1 overflow-hidden rounded-l-lg text-foreground border border-border bg-card dark:bg-surface-dark focus:ring-1 focus:ring-primary focus:border-primary h-12 p-[15px] border-r-0 text-sm font-mono"
                      readOnly
                      value={`http://127.0.0.1:${port}/config`}
                    />
                    <button
                      className="bg-card dark:bg-surface-dark border border-border border-l-0 pr-4 pl-2 rounded-r-lg flex items-center text-muted-foreground hover:text-primary transition-colors group"
                      onClick={copySubscriptionUrl}
                    >
                      <Copy size={20} className="group-active:scale-90 transition-transform" />
                    </button>
                  </div>
                </label>
                <p className="text-xs text-muted-foreground">Use this URL in your system Clash client to synchronize generated rules.</p>
              </div>
            </section>

            {/* Configuration Summary */}
            <section>
              <h2 className="text-foreground text-xl font-bold leading-tight tracking-tight pb-4">Configuration Summary</h2>
              <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                <div className="flex items-center gap-4 p-4 rounded-xl border border-border bg-card dark:bg-surface-dark">
                  <div className="size-10 rounded-lg bg-primary/10 text-primary flex items-center justify-center">
                    <FileText size={20} />
                  </div>
                  <div>
                    <p className="text-sm font-bold text-foreground">Active Profile</p>
                    <p className="text-xs text-muted-foreground">production-v2.yaml</p>
                  </div>
                </div>
                <div className="flex items-center gap-4 p-4 rounded-xl border border-border bg-card dark:bg-surface-dark">
                  <div className="size-10 rounded-lg bg-orange-500/10 text-orange-500 flex items-center justify-center">
                    <Clock size={20} />
                  </div>
                  <div>
                    <p className="text-sm font-bold text-foreground">Last Sync</p>
                    <p className="text-xs text-muted-foreground">24 minutes ago</p>
                  </div>
                </div>
              </div>
            </section>

            {serverMessage && !aiResult && (
              <div
                className={`p-4 rounded-lg text-sm ${
                  serverMessage.includes("Error") || serverMessage.includes("错误") || serverMessage.includes("失败")
                    ? "bg-destructive/10 text-destructive border border-destructive/20"
                    : "bg-emerald-500/10 text-emerald-600 border border-emerald-500/20"
                }`}
              >
                {serverMessage}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Snapshot History Sidebar */}
      <aside className="w-80 border-l border-border flex flex-col bg-card dark:bg-background overflow-hidden">
        <div className="p-6 border-b border-border">
          <div className="flex items-center justify-between">
            <h3 className="text-foreground font-bold text-base flex items-center gap-2">
              <History size={18} />
              Snapshot History
            </h3>
            <button className="text-primary text-xs font-semibold hover:underline">View All</button>
          </div>
        </div>
        <div className="flex-1 overflow-y-auto p-4 space-y-4">
          {versions.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-12 opacity-30 text-center">
              <History size={48} className="mb-2" />
              <p className="text-xs">No snapshots yet</p>
            </div>
          ) : (
            versions.map((v) => (
              <div
                key={v.id}
                className="p-4 rounded-lg bg-muted/50 border border-border hover:border-primary/50 transition-colors group cursor-pointer"
                onClick={() => handleRollback(v.id)}
              >
                <div className="flex justify-between items-start mb-2">
                  <span className="text-[10px] uppercase tracking-wider font-bold text-primary">Snapshot</span>
                  <span className="text-[10px] text-muted-foreground">{formatTimestamp(v.timestamp)}</span>
                </div>
                <p className="text-sm font-medium text-foreground group-hover:text-primary transition-colors truncate">
                  {v.description || "Untitled Snapshot"}
                </p>
                <div className="flex gap-2 mt-3 opacity-0 group-hover:opacity-100 transition-opacity">
                   <Button size="sm" variant="ghost" className="h-6 px-2 text-[10px]" onClick={(e) => { e.stopPropagation(); handleDeleteVersion(v.id); }}>
                     Delete
                   </Button>
                </div>
              </div>
            ))
          )}
        </div>
        <div className="p-6 bg-muted/30">
          <button className="w-full flex items-center justify-center gap-2 py-3 rounded-lg border border-dashed border-border text-muted-foreground hover:text-foreground hover:border-primary transition-all text-xs font-medium">
            <PlusCircle size={14} />
            Create Manual Snapshot
          </button>
        </div>
      </aside>
    </div>
  );
}
