import { useState, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogBody,
  DialogFooter,
  DialogClose,
} from "@/components/ui/dialog";
import {
  Sparkles,
  Zap,
  StopCircle,
  FileEdit,
  Copy,
  FileText,
  Clock,
  History,
  PlusCircle,
  Mic,
  MicOff,
  Paperclip,
  Network,
  PanelRightOpen,
  PanelRightClose,
  Trash2,
  RotateCcw,
  X,
  File,
  AlertCircle,
} from "lucide-react";

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

interface Subscription {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  last_updated: string | null;
  node_count: number | null;
}

interface AttachedFile {
  name: string;
  path: string;
  content: string;
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
  const [isHistoryOpen, setIsHistoryOpen] = useState(false);
  const [totalProxies, setTotalProxies] = useState<number | "N/A">("N/A");
  const [lastSync, setLastSync] = useState<string>("N/A");
  const [activeProfile, setActiveProfile] = useState<string>("N/A");
  const port = 8080;

  // New states for advanced features
  const [allSnapshotsModal, setAllSnapshotsModal] = useState(false);
  const [attachedFiles, setAttachedFiles] = useState<AttachedFile[]>([]);
  const [isRecording, setIsRecording] = useState(false);
  const [speechSupported, setSpeechSupported] = useState(false);
  const recognitionRef = useRef<SpeechRecognition | null>(null);

  useEffect(() => {
    checkServerStatus();
    loadVersions();
    loadSummaryData();
    checkSpeechSupport();
  }, []);

  function checkSpeechSupport() {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    setSpeechSupported(!!SpeechRecognition);
  }

  async function loadSummaryData() {
    try {
      const subs = await invoke<Subscription[]>("get_subscriptions");
      const count = subs.reduce((acc, sub) => acc + (sub.node_count || 0), 0);
      setTotalProxies(count || "N/A");

      const latest = subs
        .filter(s => s.last_updated)
        .sort((a, b) => new Date(b.last_updated!).getTime() - new Date(a.last_updated!).getTime())[0];

      if (latest && latest.last_updated) {
        setLastSync(formatRelativeTime(latest.last_updated));
      } else {
        setLastSync("N/A");
      }
    } catch (error) {
      console.error("Failed to load summary data:", error);
    }
  }

  function formatRelativeTime(dateStr: string) {
    const date = new Date(dateStr);
    const now = new Date();
    const diffInSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

    if (diffInSeconds < 60) return "just now";
    if (diffInSeconds < 3600) return `${Math.floor(diffInSeconds / 60)}m ago`;
    if (diffInSeconds < 86400) return `${Math.floor(diffInSeconds / 3600)}h ago`;
    return date.toLocaleDateString();
  }

  function formatTimestamp(ts: number) {
    return new Date(ts * 1000).toLocaleString();
  }

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
      if (vers.length > 0) {
        setActiveProfile(vers[0].description || "current.yaml");
      }
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
    setServerMessage("Link copied to clipboard");
    setTimeout(() => setServerMessage(""), 2000);
  }

  async function handleAiGenerate() {
    if (!aiPrompt.trim()) return;
    setAiLoading(true);
    setAiResult(null);
    try {
      // Build context with attached files
      let fullPrompt = aiPrompt;
      if (attachedFiles.length > 0) {
        const filesContext = attachedFiles
          .map(f => `--- ${f.name} ---\n${f.content}`)
          .join("\n\n");
        fullPrompt = `${aiPrompt}\n\nAttached files for context:\n${filesContext}`;
      }

      const result = await invoke<AiPatchResult>("generate_ai_patch", { prompt: fullPrompt });
      setAiResult(result);
      setServerMessage(`AI generation successful: ${result.description}`);
    } catch (error) {
      setServerMessage(`AI generation failed: ${error}`);
    } finally {
      setAiLoading(false);
    }
  }

  async function handleRollback(id: string) {
    try {
      await invoke<string>("rollback_version", { id });
      setServerMessage("Rolled back to selected version");
      loadVersions();
    } catch (error) {
      setServerMessage(`Rollback failed: ${error}`);
    }
  }

  async function handleDeleteVersion(id: string) {
    if (!confirm("Delete this snapshot?")) return;
    try {
      await invoke<string>("delete_version", { id });
      loadVersions();
    } catch (error) {
      setServerMessage(`Delete failed: ${error}`);
    }
  }

  async function handleApplyAiPatch() {
    if (!aiResult) return;
    setIsLoading(true);
    try {
      const result = await invoke<string>("apply_ai_patch", { operations: aiResult.operations });
      setServerMessage(result);
      setAiResult(null);
      setAiPrompt("");
      setAttachedFiles([]);
      loadVersions();
    } catch (error) {
      setServerMessage(`Apply failed: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function handleCreateSnapshot() {
    const desc = prompt("Enter a description for this snapshot:");
    if (desc === null) return;

    setIsLoading(true);
    try {
      await invoke("create_manual_snapshot", { description: desc || "Manual Snapshot" });
      setServerMessage("Snapshot created successfully");
      loadVersions();
    } catch (error) {
      setServerMessage(`Snapshot creation failed: ${error}`);
    } finally {
      setIsLoading(false);
    }
  }

  async function openInEditor() {
    try {
      await invoke("open_config_in_editor");
    } catch (error) {
      setServerMessage(`Cannot open editor: ${error}`);
    }
  }

  // Attach file handler
  async function handleAttachFile() {
    try {
      const filePath = await open({
        filters: [
          { name: "Config Files", extensions: ["yaml", "yml", "json", "txt"] },
        ],
        multiple: false,
      });

      if (filePath && typeof filePath === "string") {
        // Read file content (we'll need to add this command to backend)
        const content = await invoke<string>("read_file_content", { path: filePath });
        const fileName = filePath.split("/").pop() || filePath;

        setAttachedFiles(prev => [...prev, { name: fileName, path: filePath, content }]);
        setServerMessage(`Attached: ${fileName}`);
      }
    } catch (error) {
      // If read_file_content doesn't exist, just attach without content
      console.error("Could not read file:", error);
      setServerMessage(`File attachment failed: ${error}`);
    }
  }

  function removeAttachedFile(index: number) {
    setAttachedFiles(prev => prev.filter((_, i) => i !== index));
  }

  // Voice input handlers
  function toggleVoiceInput() {
    if (!speechSupported) {
      setServerMessage("Speech recognition not supported in this browser");
      return;
    }

    if (isRecording) {
      stopRecording();
    } else {
      startRecording();
    }
  }

  function startRecording() {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    const recognition = new SpeechRecognition();
    recognition.continuous = true;
    recognition.interimResults = true;
    recognition.lang = "en-US";

    recognition.onresult = (event) => {
      let transcript = "";
      for (let i = event.resultIndex; i < event.results.length; i++) {
        transcript += event.results[i][0].transcript;
      }
      setAiPrompt(prev => {
        // Replace interim results or append final
        if (event.results[event.results.length - 1].isFinal) {
          return prev + transcript + " ";
        }
        return prev;
      });
    };

    recognition.onerror = (event) => {
      console.error("Speech recognition error:", event.error);
      setIsRecording(false);
      setServerMessage(`Voice input error: ${event.error}`);
    };

    recognition.onend = () => {
      setIsRecording(false);
    };

    recognitionRef.current = recognition;
    recognition.start();
    setIsRecording(true);
  }

  function stopRecording() {
    if (recognitionRef.current) {
      recognitionRef.current.stop();
      recognitionRef.current = null;
    }
    setIsRecording(false);
  }

  return (
    <div className="flex h-full relative">
      {/* Dashboard Main Content */}
      <div className="flex-1 flex flex-col overflow-y-auto">
        {/* AI Composer Header */}
        <header className="sticky top-0 z-10 bg-background/80 backdrop-blur-md px-8 py-6">
          <div className="max-w-4xl mx-auto flex gap-4">
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

              {/* Attached Files Display */}
              {attachedFiles.length > 0 && (
                <div className="px-4 pb-2 flex flex-wrap gap-2">
                  {attachedFiles.map((file, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2 px-3 py-1.5 bg-muted rounded-lg text-xs"
                    >
                      <File size={14} className="text-primary" />
                      <span className="text-foreground font-medium">{file.name}</span>
                      <button
                        onClick={() => removeAttachedFile(index)}
                        className="text-muted-foreground hover:text-foreground"
                      >
                        <X size={14} />
                      </button>
                    </div>
                  ))}
                </div>
              )}

              <div className="flex justify-end p-4 pt-0">
                <div className="flex items-center gap-4">
                  <div className="flex items-center gap-1">
                    <button
                      className={`p-2 transition-colors ${
                        isRecording
                          ? "text-red-500 animate-pulse"
                          : "text-muted-foreground hover:text-primary"
                      }`}
                      onClick={toggleVoiceInput}
                      title={speechSupported ? "Voice input" : "Speech not supported"}
                    >
                      {isRecording ? <MicOff size={20} /> : <Mic size={20} />}
                    </button>
                    <button
                      className="p-2 text-muted-foreground hover:text-primary transition-colors"
                      onClick={handleAttachFile}
                      title="Attach file for context"
                    >
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

            <Button
              variant="outline"
              size="icon"
              className="mt-2 shrink-0 border-border"
              onClick={() => setIsHistoryOpen(!isHistoryOpen)}
            >
              {isHistoryOpen ? <PanelRightClose size={20} /> : <PanelRightOpen size={20} />}
            </Button>
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
                  <Button size="sm" onClick={handleApplyAiPatch} disabled={isLoading}>
                    Apply Changes
                  </Button>
                  <Button size="sm" variant="outline" onClick={() => setAiResult(null)} disabled={isLoading}>
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
                        onClick={openInEditor}
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
                  <div className="p-4 rounded-xl border border-border bg-card dark:bg-surface-dark flex flex-col flex-1 justify-center">
                    <span className="text-muted-foreground text-xs font-medium uppercase tracking-wider">Total Proxies</span>
                    <span className="text-2xl font-bold text-foreground mt-1">{totalProxies}</span>
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
                    <p className="text-xs text-muted-foreground truncate max-w-[200px]">{activeProfile}</p>
                  </div>
                </div>
                <div className="flex items-center gap-4 p-4 rounded-xl border border-border bg-card dark:bg-surface-dark">
                  <div className="size-10 rounded-lg bg-orange-500/10 text-orange-500 flex items-center justify-center">
                    <Clock size={20} />
                  </div>
                  <div>
                    <p className="text-sm font-bold text-foreground">Last Sync</p>
                    <p className="text-xs text-muted-foreground">{lastSync}</p>
                  </div>
                </div>
              </div>
            </section>

            {serverMessage && !aiResult && (
              <div
                className={`p-4 rounded-lg text-sm ${
                  serverMessage.includes("Error") || serverMessage.includes("failed")
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
      {isHistoryOpen && (
        <aside className="w-80 border-l border-border flex flex-col bg-card dark:bg-background overflow-hidden animate-in slide-in-from-right duration-300">
          <div className="p-6 border-b border-border">
            <div className="flex items-center justify-between">
              <h3 className="text-foreground font-bold text-base flex items-center gap-2">
                <History size={18} />
                Snapshot History
              </h3>
              <button
                className="text-primary text-xs font-semibold hover:underline"
                onClick={() => setAllSnapshotsModal(true)}
              >
                View All
              </button>
            </div>
          </div>
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {versions.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 opacity-30 text-center">
                <History size={48} className="mb-2" />
                <p className="text-xs">No snapshots yet</p>
              </div>
            ) : (
              versions.slice(0, 5).map((v) => (
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
            <button
              className="w-full flex items-center justify-center gap-2 py-3 rounded-lg border border-dashed border-border text-muted-foreground hover:text-foreground hover:border-primary transition-all text-xs font-medium"
              onClick={handleCreateSnapshot}
              disabled={isLoading}
            >
              <PlusCircle size={14} />
              Create Manual Snapshot
            </button>
          </div>
        </aside>
      )}

      {/* All Snapshots Modal */}
      <Dialog open={allSnapshotsModal} onOpenChange={setAllSnapshotsModal}>
        <DialogContent className="max-w-2xl">
          <DialogClose onClose={() => setAllSnapshotsModal(false)} />
          <DialogHeader>
            <DialogTitle>All Configuration Snapshots</DialogTitle>
            <DialogDescription>View and manage all your configuration history</DialogDescription>
          </DialogHeader>
          <DialogBody className="max-h-[500px] overflow-y-auto">
            {versions.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-16 text-muted-foreground">
                <History size={64} className="opacity-30 mb-4" />
                <p className="text-sm font-medium">No snapshots yet</p>
                <p className="text-xs mt-1">Create a snapshot to preserve your current configuration</p>
              </div>
            ) : (
              <div className="space-y-3">
                {versions.map((v, index) => (
                  <div
                    key={v.id}
                    className={`p-4 rounded-lg border transition-colors ${
                      index === 0 ? "border-primary/50 bg-primary/5" : "border-border bg-muted/30"
                    }`}
                  >
                    <div className="flex items-start justify-between gap-4">
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2 mb-1">
                          {index === 0 && (
                            <span className="px-2 py-0.5 bg-primary/10 text-primary text-[10px] font-bold rounded uppercase">
                              Latest
                            </span>
                          )}
                          <span className="text-[10px] text-muted-foreground">
                            {formatTimestamp(v.timestamp)}
                          </span>
                        </div>
                        <p className="font-bold text-foreground truncate">{v.description || "Untitled Snapshot"}</p>
                        <p className="text-xs text-muted-foreground font-mono mt-1 truncate">{v.file_path}</p>
                      </div>
                      <div className="flex items-center gap-2 shrink-0">
                        <Button
                          size="sm"
                          variant="outline"
                          className="h-8 gap-1"
                          onClick={() => {
                            handleRollback(v.id);
                            setAllSnapshotsModal(false);
                          }}
                        >
                          <RotateCcw size={14} />
                          Restore
                        </Button>
                        <Button
                          size="sm"
                          variant="ghost"
                          className="h-8 text-muted-foreground hover:text-destructive"
                          onClick={() => handleDeleteVersion(v.id)}
                        >
                          <Trash2 size={14} />
                        </Button>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => setAllSnapshotsModal(false)}>
              Close
            </Button>
            <Button onClick={() => { handleCreateSnapshot(); }} className="gap-2">
              <PlusCircle size={16} />
              Create Snapshot
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

// TypeScript declarations for Web Speech API
declare global {
  interface Window {
    SpeechRecognition: typeof SpeechRecognition;
    webkitSpeechRecognition: typeof SpeechRecognition;
  }
}
