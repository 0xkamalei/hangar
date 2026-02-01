import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Cpu, Server, ClipboardList, Sliders, Eye, EyeOff, Save, Loader2, RefreshCw, Plus, ChevronDown, FolderOpen, Zap, Trash2 } from "lucide-react";

interface LlmConfig {
  base_url: string;
  api_key: string;
  model: string;
}

interface ServerConfig {
  port: number;
  host: string;
}

interface HangarConfig {
  llm: LlmConfig;
  server: ServerConfig;
  rule_sources: string[];
}

export default function SettingsTab() {
  const { t, i18n } = useTranslation();
  const [llmConfig, setLlmConfig] = useState({
    baseUrl: "https://api.openai.com/v1",
    apiKey: "",
    model: "gpt-4o",
  });
  const [serverConfig, setServerConfig] = useState({
    port: "8080",
    host: "127.0.0.1",
  });
  const [ruleSources, setRuleSources] = useState<string[]>([]);
  const [newRuleUrl, setNewRuleUrl] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
  const [showApiKey, setShowApiKey] = useState(false);
  const [message, setMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  // Load config on mount
  useEffect(() => {
    loadConfig();
  }, []);

  async function loadConfig() {
    setIsLoading(true);
    try {
      const config = await invoke<HangarConfig>("get_hangar_config");
      setLlmConfig({
        baseUrl: config.llm.base_url,
        apiKey: config.llm.api_key,
        model: config.llm.model,
      });
      setServerConfig({
        port: String(config.server.port),
        host: config.server.host,
      });
      setRuleSources(config.rule_sources);
    } catch (error) {
      console.error("Failed to load config:", error);
      setMessage({ type: "error", text: t("settings.load_failed") });
    } finally {
      setIsLoading(false);
    }
  }

  async function saveConfig() {
    setIsSaving(true);
    setMessage(null);
    try {
      const config: HangarConfig = {
        llm: {
          base_url: llmConfig.baseUrl,
          api_key: llmConfig.apiKey,
          model: llmConfig.model,
        },
        server: {
          port: parseInt(serverConfig.port, 10) || 8080,
          host: serverConfig.host,
        },
        rule_sources: ruleSources,
      };
      await invoke<string>("save_hangar_config", { config });
      setMessage({ type: "success", text: t("settings.save_success") });
    } catch (error) {
      console.error("Failed to save config:", error);
      setMessage({ type: "error", text: t("settings.save_failed") });
    } finally {
      setIsSaving(false);
    }
  }

  function changeLanguage(lang: string) {
    i18n.changeLanguage(lang);
  }

  async function testLlmConnection() {
    setIsTesting(true);
    setMessage(null);
    try {
      const result = await invoke<string>("test_llm_connection", {
        baseUrl: llmConfig.baseUrl,
        apiKey: llmConfig.apiKey,
        model: llmConfig.model,
      });
      setMessage({ type: "success", text: result });
    } catch (error) {
      console.error("LLM connection test failed:", error);
      setMessage({ type: "error", text: String(error) });
    } finally {
      setIsTesting(false);
    }
  }

  const handleAddRuleSource = () => {
    if (newRuleUrl.trim()) {
      setRuleSources([...ruleSources, newRuleUrl.trim()]);
      setNewRuleUrl("");
    }
  };

  const handleRemoveRuleSource = (index: number) => {
    setRuleSources(ruleSources.filter((_, i) => i !== index));
  };

  return (
    <div className="flex flex-col min-h-full">
      {/* Header */}
      <header className="sticky top-0 z-10 flex items-center justify-between px-8 py-4 bg-background/80 backdrop-blur-md border-b border-border">
        <h2 className="text-xl font-bold text-foreground">Application Settings</h2>
        <div className="flex gap-3">
          <Button
            variant="ghost"
            className="text-muted-foreground hover:bg-muted"
            onClick={loadConfig}
            disabled={isLoading}
          >
            Discard
          </Button>
          <Button
            className="bg-primary text-white shadow-lg shadow-primary/20"
            onClick={saveConfig}
            disabled={isSaving || isLoading}
          >
            {isSaving ? <Loader2 className="h-4 w-4 mr-2 animate-spin" /> : <Save className="h-4 w-4 mr-2" />}
            Save Changes
          </Button>
        </div>
      </header>

      <main className="flex-1 overflow-y-auto bg-muted/20">
        <div className="max-w-4xl mx-auto p-8 space-y-10">
          {/* Status Message */}
          {message && (
            <div
              className={`p-4 rounded-xl border flex items-center gap-3 animate-in fade-in slide-in-from-top-2 ${
                message.type === "success"
                  ? "bg-emerald-500/10 text-emerald-600 border-emerald-500/20"
                  : "bg-destructive/10 text-destructive border-destructive/20"
              }`}
            >
              <div className={`size-2 rounded-full ${message.type === "success" ? "bg-emerald-500" : "bg-red-500"}`} />
              <p className="text-sm font-medium">{message.text}</p>
            </div>
          )}

          {/* 1. LLM Config Section */}
          <section>
            <div className="flex items-center gap-2 mb-6 text-foreground">
              <Cpu className="text-primary" size={20} />
              <h3 className="text-lg font-bold">LLM Configuration</h3>
            </div>
            <div className="bg-card dark:bg-surface-dark border border-border rounded-xl overflow-hidden shadow-sm">
              <div className="p-6 grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className="text-sm font-semibold text-muted-foreground">Base URL</label>
                  <Input
                    className="bg-muted/50 border-border focus:ring-1 focus:ring-primary h-11 px-4"
                    placeholder="https://api.openai.com/v1"
                    value={llmConfig.baseUrl}
                    onChange={(e) => setLlmConfig({ ...llmConfig, baseUrl: e.target.value })}
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-semibold text-muted-foreground">Model Selection</label>
                  <div className="relative">
                    <select
                      className="w-full appearance-none bg-muted/50 border border-border rounded-lg px-4 py-2.5 text-sm focus:border-primary focus:ring-1 focus:ring-primary transition-all outline-none"
                      value={llmConfig.model}
                      onChange={(e) => setLlmConfig({ ...llmConfig, model: e.target.value })}
                    >
                      <option value="gpt-4o">gpt-4o</option>
                      <option value="gpt-4-turbo">gpt-4-turbo</option>
                      <option value="gpt-3.5-turbo">gpt-3.5-turbo</option>
                      <option value="claude-3-5-sonnet">claude-3-5-sonnet</option>
                    </select>
                    <ChevronDown className="absolute right-3 top-2.5 text-muted-foreground pointer-events-none" size={18} />
                  </div>
                </div>
                <div className="space-y-2 md:col-span-2">
                  <label className="text-sm font-semibold text-muted-foreground">API Key</label>
                  <div className="relative flex items-center">
                    <Input
                      className="bg-muted/50 border-border focus:ring-1 focus:ring-primary h-11 px-4"
                      type={showApiKey ? "text" : "password"}
                      placeholder="sk-••••••••••••••••••••••••"
                      value={llmConfig.apiKey}
                      onChange={(e) => setLlmConfig({ ...llmConfig, apiKey: e.target.value })}
                    />
                    <button
                      className="absolute right-3 text-muted-foreground hover:text-foreground transition-colors"
                      onClick={() => setShowApiKey(!showApiKey)}
                    >
                      {showApiKey ? <EyeOff size={18} /> : <Eye size={18} />}
                    </button>
                  </div>
                </div>
              </div>
              <div className="px-6 py-4 bg-muted/30 border-t border-border flex justify-between items-center">
                <p className="text-xs text-muted-foreground">Configuration is used for smart rule generation and subscription parsing.</p>
                <Button
                  variant="outline"
                  size="sm"
                  className="bg-primary/5 text-primary border-primary/20 hover:bg-primary/10 font-semibold gap-2"
                  onClick={testLlmConnection}
                  disabled={isTesting}
                >
                  {isTesting ? <Loader2 className="size-3 animate-spin" /> : <Zap size={14} />}
                  Test Connection
                </Button>
              </div>
            </div>
          </section>

          {/* 2. Server Config Section */}
          <section>
            <div className="flex items-center gap-2 mb-6 text-foreground">
              <Server className="text-primary" size={20} />
              <h3 className="text-lg font-bold">Server Configuration</h3>
            </div>
            <div className="bg-card dark:bg-surface-dark border border-border rounded-xl p-6 shadow-sm">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="space-y-2">
                  <label className="text-sm font-semibold text-muted-foreground">Host</label>
                  <Input
                    className="bg-muted/50 border-border focus:ring-1 focus:ring-primary h-11 font-mono"
                    value={serverConfig.host}
                    onChange={(e) => setServerConfig({ ...serverConfig, host: e.target.value })}
                  />
                </div>
                <div className="space-y-2">
                  <label className="text-sm font-semibold text-muted-foreground">Port</label>
                  <Input
                    className="bg-muted/50 border-border focus:ring-1 focus:ring-primary h-11 font-mono"
                    type="number"
                    value={serverConfig.port}
                    onChange={(e) => setServerConfig({ ...serverConfig, port: e.target.value })}
                  />
                </div>
              </div>
            </div>
          </section>

          {/* 3. Rule Sources Section */}
          <section>
            <div className="flex items-center justify-between mb-6">
              <div className="flex items-center gap-2 text-foreground">
                <ClipboardList className="text-primary" size={20} />
                <h3 className="text-lg font-bold">Rule Sources</h3>
              </div>
              <Button variant="outline" size="sm" className="font-bold gap-2">
                <RefreshCw size={14} />
                Refresh All
              </Button>
            </div>
            <div className="bg-card dark:bg-surface-dark border border-border rounded-xl overflow-hidden shadow-sm">
              <div className="divide-y divide-border">
                {ruleSources.map((url, index) => (
                  <div key={index} className="p-4 flex items-center justify-between hover:bg-muted/10 transition-colors">
                    <div className="flex items-center gap-4">
                      <ClipboardList className="text-muted-foreground" size={18} />
                      <div className="min-w-0">
                        <p className="text-sm font-medium text-foreground truncate">{url.split('/').pop()}</p>
                        <p className="text-xs text-muted-foreground font-mono truncate">{url}</p>
                      </div>
                    </div>
                    <div className="flex items-center gap-2">
                      <span className="px-2 py-0.5 bg-emerald-500/10 text-emerald-500 text-[10px] font-bold rounded border border-emerald-500/20 uppercase">Active</span>
                      <button
                        className="p-2 text-muted-foreground hover:text-destructive transition-colors"
                        onClick={() => handleRemoveRuleSource(index)}
                      >
                        <Trash2 size={18} />
                      </button>
                    </div>
                  </div>
                ))}
              </div>
              <div className="p-4 bg-muted/30 border-t border-border">
                <div className="flex gap-2">
                  <Input
                    className="flex-1 bg-card border-border h-10 px-4 text-sm focus:ring-primary"
                    placeholder="Paste new rule set URL here..."
                    value={newRuleUrl}
                    onChange={(e) => setNewRuleUrl(e.target.value)}
                  />
                  <Button className="bg-primary text-white font-bold h-10 gap-2 px-4 shadow-sm" onClick={handleAddRuleSource}>
                    <Plus size={16} />
                    Add URL
                  </Button>
                </div>
              </div>
            </div>
          </section>

          {/* 4. App Settings Section */}
          <section className="pb-12">
            <div className="flex items-center gap-2 mb-6 text-foreground">
              <Sliders className="text-primary" size={20} />
              <h3 className="text-lg font-bold">App Settings</h3>
            </div>
            <div className="bg-card dark:bg-surface-dark border border-border rounded-xl p-6 space-y-8 shadow-sm">
              <div className="flex items-center justify-between">
                <div>
                  <p className="text-sm font-bold text-foreground">Interface Language</p>
                  <p className="text-xs text-muted-foreground">Switch between English and Chinese</p>
                </div>
                <div className="flex p-1 bg-muted rounded-lg">
                  <button
                    className={`px-4 py-1.5 text-xs font-bold rounded-md transition-all ${i18n.language === 'en' ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}`}
                    onClick={() => changeLanguage('en')}
                  >
                    English
                  </button>
                  <button
                    className={`px-4 py-1.5 text-xs font-bold rounded-md transition-all ${i18n.language === 'zh-CN' ? 'bg-card text-foreground shadow-sm' : 'text-muted-foreground hover:text-foreground'}`}
                    onClick={() => changeLanguage('zh-CN')}
                  >
                    中文 (CN)
                  </button>
                </div>
              </div>
              <div className="space-y-3">
                <div>
                  <p className="text-sm font-bold text-foreground">Data Directory</p>
                  <p className="text-xs text-muted-foreground">Local path where Hangar stores profiles and logs</p>
                </div>
                <div className="flex gap-2">
                  <Input
                    className="flex-1 bg-muted/30 border-border h-11 px-4 text-xs font-mono text-muted-foreground"
                    readOnly
                    value="~/.hangar/"
                  />
                  <Button variant="outline" className="h-11 px-4 font-bold gap-2">
                    <FolderOpen size={18} />
                    Browse
                  </Button>
                </div>
              </div>
            </div>
          </section>
        </div>
      </main>
    </div>
  );
}
