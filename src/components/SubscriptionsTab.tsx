import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
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
  PlusCircle,
  RefreshCw,
  Download,
  Upload,
  Rss,
  Eye,
  Edit2,
  Trash2,
  Search,
  Bell,
  ChevronLeft,
  ChevronRight,
  CheckCircle,
  Copy,
  X,
  AlertCircle,
  Info,
} from "lucide-react";

interface Subscription {
  id: string;
  name: string;
  url: string;
  enabled: boolean;
  lastUpdated?: string;
  nodeCount?: number;
  status: "Online" | "Error" | "Refreshing" | "Disabled";
  errorMessage?: string;
}

interface Notification {
  id: string;
  title: string;
  message: string;
  timestamp: number;
  is_read: boolean;
  severity: string;
}

export default function SubscriptionsTab() {
  const { t } = useTranslation();
  const [subscriptions, setSubscriptions] = useState<Subscription[]>([]);
  const [_showAddForm, setShowAddForm] = useState(false);
  const [editIndex, setEditIndex] = useState<number | null>(null);
  const [formData, setFormData] = useState({ name: "", url: "" });
  const [message, setMessage] = useState("");
  const [messageType, setMessageType] = useState<"success" | "error">("success");
  const [searchTerm, setSearchTerm] = useState("");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [currentPage, setCurrentPage] = useState(1);
  const itemsPerPage = 10;
  const [selectedIndices, setSelectedIndices] = useState<number[]>([]);

  // Modal states
  const [viewUrlModal, setViewUrlModal] = useState<{ open: boolean; sub: Subscription | null }>({
    open: false,
    sub: null,
  });
  const [importExportModal, setImportExportModal] = useState(false);
  const [notificationPanel, setNotificationPanel] = useState(false);
  const [notifications, setNotifications] = useState<Notification[]>([]);

  useEffect(() => {
    loadSubscriptions();
    loadNotifications();
  }, []);

  useEffect(() => {
    setCurrentPage(1);
    setSelectedIndices([]);
  }, [searchTerm]);

  async function loadNotifications() {
    try {
      const notifs = await invoke<Notification[]>("get_notifications");
      setNotifications(notifs);
    } catch (error) {
      console.error("Failed to load notifications:", error);
    }
  }

  async function fetchSubscriptionDetails(sub: Subscription): Promise<Subscription> {
    if (!sub.enabled) {
      return { ...sub, status: "Disabled", errorMessage: undefined };
    }
    try {
      // Using the refresh_subscription command which handles caching and metadata
      const updatedSub = await invoke<Subscription>("refresh_subscription", { id: sub.id });
      return { ...sub, ...updatedSub, status: "Online", errorMessage: undefined };
    } catch (error) {
      console.error(`Failed to fetch subscription details for ${sub.name}:`, error);
      return { ...sub, status: "Error", errorMessage: String(error) };
    }
  }

  async function handleRefreshSingle(id: string) {
    setSubscriptions(prev => prev.map(s => s.id === id ? { ...s, status: "Refreshing" } : s));
    try {
      const updatedSub = await invoke<Subscription>("refresh_subscription", { id });
      setSubscriptions(prev => prev.map(s => s.id === id ? { ...s, ...updatedSub, status: "Online" } : s));
      showMessage(t("common.success"), "success");
    } catch (error) {
      setSubscriptions(prev => prev.map(s => s.id === id ? { ...s, status: "Error", errorMessage: String(error) } : s));
      showMessage(`${t("common.error")}: ${error}`, "error");
    }
  }

  async function loadSubscriptions() {
    try {
      const subs = await invoke<Subscription[]>("get_subscriptions");
      const subsWithDetails = await Promise.all(
        subs.map(async (sub) => {
          return await fetchSubscriptionDetails(sub);
        })
      );
      setSubscriptions(subsWithDetails);
    } catch (error) {
      console.error("Failed to load subscriptions:", error);
      showMessage(`${t("common.error")}: ${error}`, "error");
    }
  }

  async function handleRefreshAll() {
    setIsRefreshing(true);
    const refreshedSubscriptions = await Promise.all(
      subscriptions.map(async (sub) => {
        const updatedSub = { ...sub, status: "Refreshing" as const };
        setSubscriptions((prevSubs) =>
          prevSubs.map((s) => (s.id === sub.id ? updatedSub : s))
        );
        return await fetchSubscriptionDetails(sub);
      })
    );
    setSubscriptions(refreshedSubscriptions);
    setIsRefreshing(false);
    showMessage(t("subscriptions.refreshed_all"), "success");
  }

  function showMessage(msg: string, type: "success" | "error" = "success") {
    setMessage(msg);
    setMessageType(type);
    setTimeout(() => setMessage(""), 3000);
  }

  async function handleAdd() {
    if (!formData.name || !formData.url) {
      showMessage(t("common.error") + ": Please fill in all fields", "error");
      return;
    }

    try {
      const newSub: Subscription = await invoke("add_subscription", {
        name: formData.name,
        url: formData.url,
      });
      setFormData({ name: "", url: "" });
      setShowAddForm(false);
      const subWithDetails = await fetchSubscriptionDetails(newSub);
      setSubscriptions((prevSubs) => [...prevSubs, subWithDetails]);
      showMessage(t("common.success"), "success");
    } catch (error) {
      showMessage(`${t("common.error")}: ${error}`, "error");
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
      showMessage(t("common.success"), "success");
    } catch (error) {
      showMessage(`${t("common.error")}: ${error}`, "error");
    }
  }

  async function handleDelete(index: number) {
    if (!confirm(t("subscriptions.confirm_delete"))) return;

    try {
      await invoke<string>("delete_subscription", { index });
      await loadSubscriptions();
      showMessage(t("common.success"), "success");
    } catch (error) {
      showMessage(`${t("common.error")}: ${error}`, "error");
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
      showMessage(`${t("common.error")}: ${error}`, "error");
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

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
    showMessage("Copied to clipboard!", "success");
  }

  // Import/Export handlers
  async function handleExport() {
    try {
      const filePath = await save({
        filters: [{ name: "JSON", extensions: ["json"] }],
        defaultPath: "hangar-subscriptions.json",
      });
      if (filePath) {
        await invoke("export_subscriptions", { path: filePath });
        showMessage("Subscriptions exported successfully!", "success");
        setImportExportModal(false);
      }
    } catch (error) {
      showMessage(`Export failed: ${error}`, "error");
    }
  }

  async function handleImport() {
    try {
      const filePath = await open({
        filters: [{ name: "JSON", extensions: ["json"] }],
        multiple: false,
      });
      if (filePath) {
        await invoke<Subscription[]>("import_subscriptions", { path: filePath });
        await loadSubscriptions();
        showMessage("Subscriptions imported successfully!", "success");
        setImportExportModal(false);
      }
    } catch (error) {
      showMessage(`Import failed: ${error}`, "error");
    }
  }

  // Notification handlers
  async function markNotificationRead(id: string) {
    await invoke("mark_notification_read", { id });
    loadNotifications();
  }

  async function clearAllNotifications() {
    await invoke("clear_notifications");
    setNotifications([]);
  }

  const filteredSubscriptions = subscriptions.filter(sub =>
    sub.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    sub.url.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const totalPages = Math.max(1, Math.ceil(filteredSubscriptions.length / itemsPerPage));
  const paginatedSubscriptions = filteredSubscriptions.slice(
    (currentPage - 1) * itemsPerPage,
    currentPage * itemsPerPage
  );

  function toggleSelectAll() {
    if (selectedIndices.length === paginatedSubscriptions.length) {
      setSelectedIndices([]);
    } else {
      const pageIndices = paginatedSubscriptions.map((_, i) => (currentPage - 1) * itemsPerPage + i);
      setSelectedIndices(pageIndices);
    }
  }

  function toggleSelect(index: number) {
    setSelectedIndices(prev =>
      prev.includes(index) ? prev.filter(i => i !== index) : [...prev, index]
    );
  }

  async function handleBatchDelete() {
    if (selectedIndices.length === 0) return;
    if (!confirm(`Delete ${selectedIndices.length} selected subscriptions?`)) return;

    try {
      await invoke("batch_delete_subscriptions", { indices: selectedIndices });
      setSelectedIndices([]);
      await loadSubscriptions();
      showMessage("Batch delete successful!", "success");
    } catch (error) {
      showMessage(`Batch delete failed: ${error}`, "error");
    }
  }

  async function handleBatchToggle(enabled: boolean) {
    if (selectedIndices.length === 0) return;
    try {
      await invoke("batch_toggle_subscriptions", { indices: selectedIndices, enabled });
      setSelectedIndices([]);
      await loadSubscriptions();
      showMessage(`${enabled ? "Enabled" : "Disabled"} selected items`, "success");
    } catch (error) {
      showMessage(`Operation failed: ${error}`, "error");
    }
  }

  const unreadCount = notifications.filter(n => !n.is_read).length;

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
          <button
            className="relative flex items-center justify-center rounded-lg h-10 w-10 bg-muted text-muted-foreground hover:bg-muted/80 transition-colors"
            onClick={() => setNotificationPanel(true)}
          >
            <Bell size={20} />
            {unreadCount > 0 && (
              <span className="absolute -top-1 -right-1 size-5 bg-red-500 text-white text-[10px] font-bold rounded-full flex items-center justify-center">
                {unreadCount}
              </span>
            )}
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
              <Button
                variant="outline"
                className="h-11 px-4 font-bold gap-2"
                onClick={handleRefreshAll}
                disabled={isRefreshing}
              >
                <RefreshCw size={18} className={isRefreshing ? "animate-spin" : ""} />
                <span>{isRefreshing ? "Refreshing..." : "Refresh All"}</span>
              </Button>
              <Button
                variant="outline"
                className="h-11 px-4 font-bold gap-2"
                onClick={() => setImportExportModal(true)}
              >
                <Download size={18} />
                <span>Import/Export</span>
              </Button>
            </div>
          </div>

          {/* Add/Edit Form Section */}
          <div className="bg-card dark:bg-surface-dark rounded-xl border border-border p-6 mb-8 shadow-sm">
            {selectedIndices.length > 0 ? (
              <div className="flex items-center justify-between animate-in fade-in slide-in-from-left-2">
                <div className="flex items-center gap-4">
                  <span className="text-sm font-bold text-primary">{selectedIndices.length} items selected</span>
                  <div className="h-4 w-px bg-border mx-2" />
                  <div className="flex gap-2">
                    <Button size="sm" variant="outline" className="h-9" onClick={() => handleBatchToggle(true)}>Enable All</Button>
                    <Button size="sm" variant="outline" className="h-9" onClick={() => handleBatchToggle(false)}>Disable All</Button>
                    <Button size="sm" variant="destructive" className="h-9" onClick={handleBatchDelete}>Delete All</Button>
                  </div>
                </div>
                <Button size="sm" variant="ghost" onClick={() => setSelectedIndices([])}>Cancel Selection</Button>
              </div>
            ) : (
              <>
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
                  </div>
                </div>
              </>
            )}
          </div>

          {/* Subscriptions Table */}
          <div className="bg-card dark:bg-surface-dark rounded-xl border border-border overflow-hidden shadow-sm">
            <div className="overflow-x-auto">
              <table className="w-full text-left border-collapse">
                <thead>
                  <tr className="border-b border-border bg-muted/30">
                    <th className="px-6 py-4 w-10">
                      <input
                        type="checkbox"
                        className="rounded border-border text-primary focus:ring-primary"
                        checked={paginatedSubscriptions.length > 0 && selectedIndices.length === paginatedSubscriptions.length}
                        onChange={toggleSelectAll}
                      />
                    </th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Name</th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">URL</th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-center">Nodes</th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider">Status</th>
                    <th className="px-6 py-4 text-xs font-bold text-muted-foreground uppercase tracking-wider text-right">Actions</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-border">
                  {paginatedSubscriptions.length === 0 ? (
                    <tr>
                      <td colSpan={6} className="px-6 py-12 text-center text-muted-foreground">
                        No subscriptions found.
                      </td>
                    </tr>
                  ) : (
                    paginatedSubscriptions.map((sub, index) => {
                      const actualIndex = (currentPage - 1) * itemsPerPage + index;
                      const isSelected = selectedIndices.includes(actualIndex);
                      return (
                      <tr key={sub.id || actualIndex} className={`hover:bg-muted/10 transition-colors ${isSelected ? 'bg-primary/5' : ''}`}>
                        <td className="px-6 py-5">
                          <input
                            type="checkbox"
                            className="rounded border-border text-primary focus:ring-primary"
                            checked={isSelected}
                            onChange={() => toggleSelect(actualIndex)}
                          />
                        </td>
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
                            <button
                              className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-primary"
                              onClick={() => setViewUrlModal({ open: true, sub })}
                            >
                              <Eye size={16} />
                            </button>
                          </div>
                        </td>
                        <td className="px-6 py-5 text-center">
                          <div className="flex flex-col items-center gap-1">
                            <span className="bg-primary/10 text-primary px-3 py-1 rounded-full text-xs font-bold">
                              {sub.nodeCount || 0} Nodes
                            </span>
                            {sub.lastUpdated && (
                              <span className="text-[10px] text-muted-foreground whitespace-nowrap">
                                {sub.lastUpdated}
                              </span>
                            )}
                          </div>
                        </td>
                        <td className="px-6 py-5">
                          <div className="flex items-center gap-2">
                            <div className={`size-2 rounded-full ${
                              sub.status === "Refreshing" ? "bg-blue-500 animate-pulse" :
                              sub.status === "Online" ? "bg-emerald-500" :
                              sub.status === "Error" ? "bg-red-500" : "bg-muted-foreground"
                            }`}></div>
                            <span className={`text-sm font-medium ${
                              sub.status === "Refreshing" ? "text-blue-500" :
                              sub.status === "Online" ? "text-emerald-500" :
                              sub.status === "Error" ? "text-red-500" : "text-muted-foreground"
                            }`}>
                              {sub.status || (sub.enabled ? 'Online' : 'Disabled')}
                            </span>
                          </div>
                        </td>
                        <td className="px-6 py-5">
                          <div className="flex items-center justify-end gap-3">
                            <button
                              onClick={() => handleRefreshSingle(sub.id)}
                              disabled={sub.status === "Refreshing"}
                              className={`p-1.5 rounded-md hover:bg-muted text-muted-foreground hover:text-primary transition-colors ${sub.status === "Refreshing" ? "animate-spin text-primary" : ""}`}
                              title="Refresh"
                            >
                              <RefreshCw size={18} />
                            </button>
                            <button
                              onClick={() => handleToggle(actualIndex)}
                              className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${sub.enabled ? 'bg-primary' : 'bg-muted-foreground/30'}`}
                            >
                              <span className={`inline-block h-4 w-4 transform rounded-full bg-white transition ${sub.enabled ? 'translate-x-6' : 'translate-x-1'}`} />
                            </button>
                            <button
                              onClick={() => startEdit(actualIndex)}
                              className="p-1.5 rounded-md hover:bg-muted text-muted-foreground hover:text-primary transition-colors"
                            >
                              <Edit2 size={18} />
                            </button>
                            <button
                              onClick={() => handleDelete(actualIndex)}
                              className="p-1.5 rounded-md hover:bg-destructive/10 text-muted-foreground hover:text-destructive transition-colors"
                            >
                              <Trash2 size={18} />
                            </button>
                          </div>
                        </td>
                      </tr>
                    )})
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
                <button
                  className="size-8 flex items-center justify-center rounded bg-card border border-border text-muted-foreground hover:text-primary transition-colors disabled:opacity-50"
                  onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
                  disabled={currentPage === 1}
                >
                  <ChevronLeft size={18} />
                </button>
                <div className="flex items-center px-2 text-xs font-bold text-foreground">
                  {currentPage} / {totalPages}
                </div>
                <button
                  className="size-8 flex items-center justify-center rounded bg-card border border-border text-muted-foreground hover:text-primary transition-colors disabled:opacity-50"
                  onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
                  disabled={currentPage === totalPages}
                >
                  <ChevronRight size={18} />
                </button>
              </div>
            </div>
          </div>
        </div>
      </main>

      {/* View URL Modal */}
      <Dialog open={viewUrlModal.open} onOpenChange={(open) => setViewUrlModal({ open, sub: viewUrlModal.sub })}>
        <DialogContent>
          <DialogClose onClose={() => setViewUrlModal({ open: false, sub: null })} />
          <DialogHeader>
            <DialogTitle>Subscription URL</DialogTitle>
            <DialogDescription>{viewUrlModal.sub?.name}</DialogDescription>
          </DialogHeader>
          <DialogBody>
            <div className="bg-muted rounded-lg p-4 font-mono text-sm break-all select-all">
              {viewUrlModal.sub?.url}
            </div>
          </DialogBody>
          <DialogFooter>
            <Button variant="outline" onClick={() => setViewUrlModal({ open: false, sub: null })}>
              Close
            </Button>
            <Button onClick={() => viewUrlModal.sub && copyToClipboard(viewUrlModal.sub.url)} className="gap-2">
              <Copy size={16} />
              Copy URL
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Import/Export Modal */}
      <Dialog open={importExportModal} onOpenChange={setImportExportModal}>
        <DialogContent>
          <DialogClose onClose={() => setImportExportModal(false)} />
          <DialogHeader>
            <DialogTitle>Import / Export Subscriptions</DialogTitle>
            <DialogDescription>Backup or restore your subscription list</DialogDescription>
          </DialogHeader>
          <DialogBody className="space-y-4">
            <div className="p-4 rounded-lg border border-border bg-muted/30 space-y-3">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-primary/10 text-primary">
                  <Download size={20} />
                </div>
                <div>
                  <p className="font-bold text-foreground">Export Subscriptions</p>
                  <p className="text-xs text-muted-foreground">Save all subscriptions to a JSON file</p>
                </div>
              </div>
              <Button onClick={handleExport} className="w-full gap-2">
                <Download size={16} />
                Export to File
              </Button>
            </div>
            <div className="p-4 rounded-lg border border-border bg-muted/30 space-y-3">
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-lg bg-emerald-500/10 text-emerald-500">
                  <Upload size={20} />
                </div>
                <div>
                  <p className="font-bold text-foreground">Import Subscriptions</p>
                  <p className="text-xs text-muted-foreground">Load subscriptions from a JSON file</p>
                </div>
              </div>
              <Button onClick={handleImport} variant="outline" className="w-full gap-2">
                <Upload size={16} />
                Import from File
              </Button>
            </div>
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* Notification Panel */}
      <Dialog open={notificationPanel} onOpenChange={setNotificationPanel}>
        <DialogContent className="max-w-md">
          <DialogClose onClose={() => setNotificationPanel(false)} />
          <DialogHeader>
            <div className="flex items-center justify-between">
              <DialogTitle>Notifications</DialogTitle>
              {notifications.length > 0 && (
                <button
                  onClick={clearAllNotifications}
                  className="text-xs text-muted-foreground hover:text-foreground"
                >
                  Clear All
                </button>
              )}
            </div>
          </DialogHeader>
          <DialogBody className="max-h-[400px] overflow-y-auto">
            {notifications.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-muted-foreground">
                <Bell size={48} className="opacity-30 mb-4" />
                <p className="text-sm">No notifications</p>
              </div>
            ) : (
              <div className="space-y-3">
                {notifications.map((n) => (
                  <div
                    key={n.id}
                    className={`p-4 rounded-lg border transition-colors cursor-pointer ${
                      n.is_read ? 'bg-muted/30 border-border' : 'bg-primary/5 border-primary/20'
                    }`}
                    onClick={() => markNotificationRead(n.id)}
                  >
                    <div className="flex items-start gap-3">
                      <div className={`p-1.5 rounded-lg ${
                        n.severity === 'error' ? 'bg-red-500/10 text-red-500' :
                        n.severity === 'warning' ? 'bg-orange-500/10 text-orange-500' :
                        'bg-primary/10 text-primary'
                      }`}>
                        {n.severity === 'error' ? <AlertCircle size={16} /> :
                         n.severity === 'warning' ? <AlertCircle size={16} /> :
                         <Info size={16} />}
                      </div>
                      <div className="flex-1 min-w-0">
                        <p className="font-bold text-sm text-foreground">{n.title}</p>
                        <p className="text-xs text-muted-foreground mt-1">{n.message}</p>
                        <p className="text-[10px] text-muted-foreground mt-2">
                          {new Date(n.timestamp * 1000).toLocaleString()}
                        </p>
                      </div>
                      {!n.is_read && (
                        <div className="size-2 rounded-full bg-primary shrink-0" />
                      )}
                    </div>
                  </div>
                ))}
              </div>
            )}
          </DialogBody>
        </DialogContent>
      </Dialog>

      {/* Toast Notification */}
      {message && (
        <div className="fixed bottom-8 right-8 flex flex-col gap-3 z-50">
          <div className={`${messageType === 'success' ? 'bg-emerald-500' : 'bg-red-500'} text-white px-4 py-3 rounded-lg shadow-xl flex items-center gap-3 animate-in fade-in slide-in-from-right-4`}>
            {messageType === 'success' ? <CheckCircle size={20} /> : <AlertCircle size={20} />}
            <span className="text-sm font-bold">{message}</span>
          </div>
        </div>
      )}
    </div>
  );
}
