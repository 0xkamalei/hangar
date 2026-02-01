import { useState } from "react";
import { useTranslation } from "react-i18next";
import MainTab from "@/components/MainTab";
import SubscriptionsTab from "@/components/SubscriptionsTab";
import SettingsTab from "@/components/SettingsTab";
import { LayoutDashboard, Link, Settings, Plane } from "lucide-react";

function App() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState("main");

  const navItems = [
    { id: "main", label: t("tabs.main"), icon: LayoutDashboard },
    { id: "subscriptions", label: t("tabs.subscriptions"), icon: Link },
    { id: "settings", label: t("tabs.settings"), icon: Settings },
  ];

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      {/* Sidebar Navigation */}
      <aside className="w-64 border-r border-border flex flex-col bg-card dark:bg-background">
        {/* macOS Drag Region */}
        <div data-tauri-drag-region className="h-8 shrink-0" />

        <div className="p-6 flex flex-col gap-6 h-full pt-2">
          {/* Logo */}
          <div className="flex items-center gap-3">
            <div className="bg-primary rounded-lg p-2 flex items-center justify-center shadow-lg shadow-primary/20">
              <Plane className="text-white size-6" />
            </div>
            <div className="flex flex-col">
              <h1 className="text-foreground text-lg font-bold leading-tight">Hangar</h1>
              <p className="text-muted-foreground text-xs font-normal">AI Config Manager</p>
            </div>
          </div>

          {/* Navigation Links */}
          <nav className="flex flex-col gap-2 mt-4">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = activeTab === item.id;
              return (
                <button
                  key={item.id}
                  onClick={() => setActiveTab(item.id)}
                  className={`flex items-center gap-3 px-3 py-2.5 rounded-lg transition-all ${
                    isActive
                      ? "bg-primary/10 text-primary border border-primary/20"
                      : "text-muted-foreground hover:bg-muted dark:hover:bg-surface-dark"
                  }`}
                >
                  <Icon size={20} className={isActive ? "text-primary" : ""} />
                  <p className={`text-sm ${isActive ? "font-semibold" : "font-medium"}`}>
                    {item.label}
                  </p>
                </button>
              );
            })}
          </nav>

          <div className="mt-auto pt-6 border-t border-border">
            <div className="flex items-center gap-3 px-2">
              <div className="bg-primary/20 aspect-square rounded-full size-10 border-2 border-primary/30 flex items-center justify-center">
                <span className="text-primary font-bold text-xs">User</span>
              </div>
              <div className="flex flex-col">
                <p className="text-foreground text-sm font-medium">Alex Chen</p>
                <p className="text-muted-foreground text-xs">Pro Plan</p>
              </div>
            </div>
          </div>
        </div>
      </aside>

      {/* Main Content Area */}
      <main className="flex-1 flex flex-col overflow-hidden">
        {/* macOS Drag Region for Main Area */}
        <div data-tauri-drag-region className="h-8 shrink-0" />

        <div className="flex-1 overflow-auto">
          {activeTab === "main" && <MainTab />}
          {activeTab === "subscriptions" && <SubscriptionsTab />}
          {activeTab === "settings" && <SettingsTab />}
        </div>
      </main>
    </div>
  );
}

export default App;
