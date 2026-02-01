import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Globe, Server, BookOpen, Languages, FolderOpen, TestTube, Save, Loader2 } from "lucide-react";

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
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isTesting, setIsTesting] = useState(false);
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

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">{t("settings.title")}</h2>
        <Button onClick={saveConfig} disabled={isSaving || isLoading}>
          {isSaving ? (
            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
          ) : (
            <Save className="h-4 w-4 mr-2" />
          )}
          {t("settings.save")}
        </Button>
      </div>

      {/* Status Message */}
      {message && (
        <div
          className={`p-4 rounded-md ${
            message.type === "success"
              ? "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200"
              : "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200"
          }`}
        >
          {message.text}
        </div>
      )}

      {/* LLM Configuration */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Globe className="h-5 w-5" />
            {t("settings.llm_config")}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">
              {t("settings.api_base_url")}
            </label>
            <Input
              value={llmConfig.baseUrl}
              onChange={(e) =>
                setLlmConfig({ ...llmConfig, baseUrl: e.target.value })
              }
              placeholder="https://api.openai.com/v1"
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">
              {t("settings.api_key")}
            </label>
            <Input
              type="password"
              value={llmConfig.apiKey}
              onChange={(e) =>
                setLlmConfig({ ...llmConfig, apiKey: e.target.value })
              }
              placeholder="sk-..."
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">{t("settings.model")}</label>
            <Input
              value={llmConfig.model}
              onChange={(e) =>
                setLlmConfig({ ...llmConfig, model: e.target.value })
              }
              placeholder="gpt-4o"
            />
          </div>
          <Button variant="outline" onClick={testLlmConnection} disabled={isTesting}>
            {isTesting ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <TestTube className="h-4 w-4 mr-2" />
            )}
            {t("settings.test_connection")}
          </Button>
        </CardContent>
      </Card>

      {/* Server Configuration */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Server className="h-5 w-5" />
            {t("settings.server_config")}
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">{t("settings.port")}</label>
              <Input
                value={serverConfig.port}
                onChange={(e) =>
                  setServerConfig({ ...serverConfig, port: e.target.value })
                }
                placeholder="8080"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">
                {t("settings.bind_address")}
              </label>
              <Input
                value={serverConfig.host}
                onChange={(e) =>
                  setServerConfig({ ...serverConfig, host: e.target.value })
                }
                placeholder="127.0.0.1"
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Rule Library */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <BookOpen className="h-5 w-5" />
            {t("settings.rule_library")}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground mb-4">
            {t("settings.builtin_rules")}: Loyalsoldier/clash-rules
          </p>
          <Button variant="outline">
            {t("settings.refresh_rules")}
          </Button>
        </CardContent>
      </Card>

      {/* Language Settings */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Languages className="h-5 w-5" />
            {t("settings.language")}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex gap-2">
            <Button
              variant={i18n.language === "zh-CN" ? "default" : "outline"}
              onClick={() => changeLanguage("zh-CN")}
            >
              中文
            </Button>
            <Button
              variant={i18n.language === "en" ? "default" : "outline"}
              onClick={() => changeLanguage("en")}
            >
              English
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Data Directory */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FolderOpen className="h-5 w-5" />
            {t("settings.data_directory")}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <code className="text-sm bg-muted px-2 py-1 rounded">
            ~/.hangar/
          </code>
        </CardContent>
      </Card>
    </div>
  );
}
