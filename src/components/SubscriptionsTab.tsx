import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Plus, Trash2, Edit2, RefreshCw, Power } from "lucide-react";

interface Subscription {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  lastUpdated?: string;
  nodeCount?: number;
}

export default function SubscriptionsTab() {
  const { t } = useTranslation();
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editIndex, setEditIndex] = useState<number | null>(null);
  const [formData, setFormData] = useState({ name: "", url: "" });
  const [message, setMessage] = useState("");

  useEffect(() => {
    loadSubscriptions();
  }, []);

  async function loadSubscriptions() {
    try {
      const subs = await invoke<Subscription[]>("get_subscriptions");
      setSubscriptions(subs);
    } catch (error) {
      console.error("Failed to load subscriptions:", error);
    }
  }

  function showMessage(msg: string) {
    setMessage(msg);
    setTimeout(() => setMessage(""), 3000);
  }

  async function handleAdd() {
    if (!formData.name || !formData.url) {
      showMessage(t("common.error") + ": Please fill in all fields");
      return;
    }

    try {
      await invoke<string>("add_subscription", {
        name: formData.name,
        url: formData.url,
      });
      setFormData({ name: "", url: "" });
      setShowAddForm(false);
      await loadSubscriptions();
      showMessage(t("common.success"));
    } catch (error) {
      showMessage(`${t("common.error")}: ${error}`);
    }
  }

  async function handleUpdate() {
    if (editIndex === null) return;

    try {
      await invoke<string>("update_subscription", {
        index: editIndex,
        name: formData.name,
        url: formData.url,
        enabled: subscriptions[editIndex].enabled,
      });
      setEditIndex(null);
      setFormData({ name: "", url: "" });
      await loadSubscriptions();
      showMessage(t("common.success"));
    } catch (error) {
      showMessage(`${t("common.error")}: ${error}`);
    }
  }

  async function handleDelete(index: number) {
    if (!confirm(t("subscriptions.confirm_delete"))) return;

    try {
      await invoke<string>("delete_subscription", { index });
      await loadSubscriptions();
      showMessage(t("common.success"));
    } catch (error) {
      showMessage(`${t("common.error")}: ${error}`);
    }
  }

  async function handleToggle(index: number) {
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
      showMessage(`${t("common.error")}: ${error}`);
    }
  }

  function startEdit(index: number) {
    const sub = subscriptions[index];
    setEditIndex(index);
    setFormData({ name: sub.name, url: sub.url });
    setShowAddForm(false);
  }

  function cancelForm() {
    setShowAddForm(false);
    setEditIndex(null);
    setFormData({ name: "", url: "" });
  }

  return (
    <div className="p-6 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">{t("subscriptions.title")}</h2>
        <div className="flex gap-2">
          <Button variant="outline" size="sm">
            <RefreshCw className="h-4 w-4 mr-2" />
            {t("subscriptions.refresh_all")}
          </Button>
          <Button
            size="sm"
            onClick={() => {
              setShowAddForm(true);
              setEditIndex(null);
              setFormData({ name: "", url: "" });
            }}
          >
            <Plus className="h-4 w-4 mr-2" />
            {t("subscriptions.add")}
          </Button>
        </div>
      </div>

      {/* Message */}
      {message && (
        <div
          className={`p-3 rounded-lg text-sm ${
            message.includes("Error") || message.includes("错误")
              ? "bg-destructive/10 text-destructive"
              : "bg-green-500/10 text-green-700"
          }`}
        >
          {message}
        </div>
      )}

      {/* Add/Edit Form */}
      {(showAddForm || editIndex !== null) && (
        <Card>
          <CardHeader>
            <CardTitle>
              {editIndex !== null ? t("subscriptions.edit") : t("subscriptions.add")}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">
                {t("subscriptions.name")}
              </label>
              <Input
                value={formData.name}
                onChange={(e) =>
                  setFormData({ ...formData, name: e.target.value })
                }
                placeholder="机场名称"
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">
                {t("subscriptions.url")}
              </label>
              <Input
                value={formData.url}
                onChange={(e) =>
                  setFormData({ ...formData, url: e.target.value })
                }
                placeholder="https://..."
              />
            </div>
            <div className="flex gap-2">
              <Button onClick={editIndex !== null ? handleUpdate : handleAdd}>
                {t("common.save")}
              </Button>
              <Button variant="outline" onClick={cancelForm}>
                {t("common.cancel")}
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Subscription List */}
      <div className="space-y-3">
        {subscriptions.length === 0 ? (
          <Card>
            <CardContent className="py-8 text-center text-muted-foreground">
              {t("subscriptions.no_subscriptions")}
            </CardContent>
          </Card>
        ) : (
          subscriptions.map((sub, index) => (
            <Card
              key={sub.id || index}
              className={`${!sub.enabled ? "opacity-60" : ""}`}
            >
              <CardContent className="py-4">
                <div className="flex items-center justify-between">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="font-medium truncate">{sub.name}</span>
                      {sub.enabled && (
                        <span className="text-xs px-2 py-0.5 bg-green-500/10 text-green-700 rounded-full">
                          {t("subscriptions.enabled")}
                        </span>
                      )}
                    </div>
                    <p className="text-sm text-muted-foreground truncate mt-1">
                      {sub.url}
                    </p>
                  </div>
                  <div className="flex items-center gap-2 ml-4">
                    <Button
                      size="sm"
                      variant={sub.enabled ? "outline" : "default"}
                      onClick={() => handleToggle(index)}
                    >
                      <Power className="h-4 w-4" />
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={() => startEdit(index)}
                    >
                      <Edit2 className="h-4 w-4" />
                    </Button>
                    <Button
                      size="sm"
                      variant="destructive"
                      onClick={() => handleDelete(index)}
                    >
                      <Trash2 className="h-4 w-4" />
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))
        )}
      </div>
    </div>
  );
}
