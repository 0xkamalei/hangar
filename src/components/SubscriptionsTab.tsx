import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  PlusCircle,
  RefreshCw,
  Import,
  Rss,
  Eye,
  Edit2,
  Trash2,
  Search,
  Bell,
  ChevronLeft,
  ChevronRight,
  CheckCircle,
  FolderOpen
} from "lucide-react";

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
  const [_showAddForm, setShowAddForm] = useState(false);
  const [editIndex, setEditIndex] = useState<number | null>(null);
  const [formData, setFormData] = useState({ name: "", url: "" });
  const [message, setMessage] = useState("");
  const [searchTerm, setSearchTerm] = useState("");

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
    setShowAddForm(true);
  }

  function cancelForm() {
    setShowAddForm(false);
    setEditIndex(null);
    setFormData({ name: "", url: "" });
  }

  const filteredSubscriptions = subscriptions.filter(sub =>
    sub.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    sub.url.toLowerCase().includes(searchTerm.toLowerCase())
  );

  return (
    <div className="flex flex-col min-h-full">
      {/* Search Header */}
      <header className="flex items-center justify-end px-10 py-3 border-b border-border bg-card dark:bg-background sticky top-0 z-10">
        <div className="flex gap-4">
          <div className="flex w-64 items-stretch rounded-lg bg-muted border border-transparent focus-within:border-primary transition-all overflow-hidden h-10">
            <div className="text-muted-foreground flex items-center justify-center pl-3">
              <Search size={18} />
            </div>
            <input
              className="w-full border-none bg-transparent focus:ring-0 text-sm placeholder:text-muted-foreground pl-2"
              placeholder="Search subscriptions..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
          </div>
          <button className="flex items-center justify-center rounded-lg h-10 w-10 bg-muted text-muted-foreground hover:bg-muted/80 transition-colors">
            <Bell size={20} />
          </button>
        </div>
      </header>

      <main className="flex-1 overflow-y-auto px-4 lg:px-20 py-8">
        <div className="max-w-[1200px] mx-auto">
          {/* Page Heading */}
          <div className="flex flex-wrap justify-between items-end gap-4 mb-8">
            <div className="flex flex-col gap-2">
              <h1 className="text-foreground text-4xl font-black leading-tight tracking-tight">Subscription Management</h1>
              <p className="text-muted-foreground text-base font-normal max-w-2xl">Manage your proxy sources with AI-powered Clash configuration and smart rule editing for seamless connectivity.</p>
            </div>
            <div className="flex gap-3">
              <Button variant="outline" className="h-11 px-4 font-bold gap-2">
                <RefreshCw size={18} />
                <span>Refresh All</span>
              </Button>
              <Button variant="outline" className="h-11 px-4 font-bold gap-2">
                <Import size={18} />
                <span>Import/Export</span>
              </Button>
            </div>
          </div>

          {/* Add/Edit Form Section */}
          <div className="bg-card dark:bg-surface-dark rounded-xl border border-border p-6 mb-8 shadow-sm">
            <h2 className="text-foreground text-lg font-bold mb-4 flex items-center gap-2">
              <PlusCircle className="text-primary" size={20} />
              {editIndex !== null ? "Edit Subscription" : "Add New Subscription"}
            </h2>
            <div className="flex flex-col lg:flex-row items-end gap-4">
              <div className="flex flex-col flex-1 min-w-[240px] w-full">
                <p className="text-foreground text-sm font-medium pb-2">Subscription Name</p>
                <Input
                  className="h-12 bg-muted border-border focus:ring-2 focus:ring-primary"
                  placeholder="e.g. Premium Proxy"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                />
              </div>
              <div className="flex flex-col flex-[2] min-w-[320px] w-full">
                <p className="text-foreground text-sm font-medium pb-2">Subscription URL</p>
                <Input
                  className="h-12 bg-muted border-border focus:ring-2 focus:ring-primary"
                  placeholder="https://example.com/sub/..."
                  value={formData.url}
                  onChange={(e) => setFormData({ ...formData, url: e.target.value })}
                />
              </div>
              <div className="flex gap-3 w-full lg:w-auto">
                <Button
                  className="h-12 px-6 bg-primary text-white font-bold hover:bg-blue-600 flex-1 lg:flex-none"
                  onClick={editIndex !== null ? handleUpdate : handleAdd}
                >
                  {editIndex !== null ? "Update" : "Add Subscription"}
                </Button>
                {editIndex !== null && (
                   <Button variant="outline" className="h-12 px-4 font-bold" onClick={cancelForm}>
                     Cancel
                   </Button>
                )}
                <Button variant="outline" className="h-12 px-4 font-bold gap-2">
                  <FolderOpen size={18} />
                  <span>Batch</span>
                </Button>
              </div>
            </div>
          </div>

          {/* Subscriptions Table */}
          <div className="bg-card dark:bg-surface-dark rounded-xl border border-border overflow-hidden shadow-sm">
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Name</th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">URL</th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-center">Nodes</th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Status</th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {filteredSubscriptions.length === 0 ? (
                    <tr>
                      <td colSpan={5} className="px-6 py-12 text-center text-muted-foreground">
                        No subscriptions found.
                      </td>
                    </tr>
                  ) : (
                    filteredSubscriptions.map((sub, index) => (
                      <tr key={sub.id || index} className="hover:bg-muted/10 transition-colors">
                        <td className="px-6 py-5">
                          <div className="flex items-center gap-3">
                            <div className={`p-2 rounded-lg ${sub.enabled ? 'bg-primary/10 text-primary' : 'bg-muted text-muted-foreground'}`}>
                              <Rss size={20} />
                            </div>
                            <span className="font-semibold text-foreground truncate">{sub.name}</span>
                          </div>
                        </td>
                        <td className="px-6 py-5">
                          <div className="flex items-center gap-2 group max-w-xs">
                            <span className="text-muted-foreground font-mono text-sm truncate">{sub.url}</span>
                            <button className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-primary">
                              <Eye size={16} />
                            </button>
                          </div>
                        </td>
                        <td className="px-6 py-5 text-center">
                          <span className="bg-primary/10 text-primary px-3 py-1 rounded-full text-xs font-bold">
                            {sub.nodeCount || 0} Nodes
                          </span>
                        </td>
                        <td className="px-6 py-5">
                          <div className="flex items-center gap-2">
                            <div className={`size-2 rounded-full ${sub.enabled ? 'bg-emerald-500' : 'bg-muted-foreground'}`}></div>
                            <span className={`text-sm font-medium ${sub.enabled ? 'text-emerald-500' : 'text-muted-foreground'}`}>
                              {sub.enabled ? 'Online' : 'Disabled'}
                            </span>
                          </div>
                        </td>
                        <td className="px-6 py-5">
                          <div className="flex items-center justify-end gap-3">
                            {/* Toggle Switch */}
                            <button
                              onClick={() => handleToggle(index)}
                              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${sub.enabled ? 'bg-primary' : 'bg-muted-foreground/30'}`}
                            >
                              <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition ${sub.enabled ? 'translate-x-6' : 'translate-x-1'}`} />
                            </button>
                            <button
                              onClick={() => startEdit(index)}
                              className="p-1.5 rounded-md hover:bg-muted text-muted-foreground hover:text-primary transition-colors"
                            >
                              <Edit2 size={18} />
                            </button>
                            <button
                              onClick={() => handleDelete(index)}
                              className="p-1.5 rounded-md hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"
                            >
                              <Trash2 size={18} />
                            </button>
                          </div>
                        </td>
                      </tr>
                    ))
                  )}
                </tbody>
              </table>
            </div>
            {/* Table Footer */}
            <div className="px-6 py-4 bg-muted/30 border-t border-border flex justify-between items-center">
              <span className="text-xs text-muted-foreground uppercase font-bold">
                Total: {filteredSubscriptions.length} Subscriptions
              </span>
              <div className="flex gap-1">
                <button className="size-8 flex items-center justify-center rounded bg-card border border-border text-muted-foreground hover:text-primary transition-colors">
                  <ChevronLeft size={18} />
                </button>
                <button className="size-8 flex items-center justify-center rounded bg-primary text-white font-bold text-xs">1</button>
                <button className="size-8 flex items-center justify-center rounded bg-card border border-border text-muted-foreground hover:text-primary transition-colors">
                  <ChevronRight size={18} />
                </button>
              </div>
            </div>
          </div>
        </div>
      </main>

      {/* Toast Notification */}
      {message && (
        <div className="fixed bottom-8 right-8 flex flex-col gap-3 z-50">
          <div className="bg-emerald-500 text-white px-4 py-3 rounded-lg shadow-xl flex items-center gap-3 animate-in fade-in slide-in-from-right-4">
            <CheckCircle size={20} />
            <span className="text-sm font-bold">{message}</span>
          </div>
        </div>
      )}
    </div>
  );
}
