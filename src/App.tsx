import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import MainTab from "@/components/MainTab";
import SubscriptionsTab from "@/components/SubscriptionsTab";
import SettingsTab from "@/components/SettingsTab";

function App() {
  const { t } = useTranslation();
  const [activeTab, setActiveTab] = useState("main");

  return (
    <div className="h-screen flex flex-col bg-background">
      {/* macOS 原生风格拖拽区域 */}
      <div data-tauri-drag-region className="h-8 bg-background border-b border-border" />

      {/* 主内容区域 */}
      <div className="flex-1 overflow-hidden">
        <Tabs value={activeTab} onValueChange={setActiveTab} className="h-full flex flex-col">
          {/* Tab 导航 */}
          <div className="border-b border-border px-6 pt-4">
            <TabsList className="grid w-full max-w-md grid-cols-3">
              <TabsTrigger value="main" active={activeTab === "main"}>
                {t("tabs.main")}
              </TabsTrigger>
              <TabsTrigger value="subscriptions" active={activeTab === "subscriptions"}>
                {t("tabs.subscriptions")}
              </TabsTrigger>
              <TabsTrigger value="settings" active={activeTab === "settings"}>
                {t("tabs.settings")}
              </TabsTrigger>
            </TabsList>
          </div>

          {/* Tab 内容 */}
          <div className="flex-1 overflow-auto">
            <TabsContent value="main" className="h-full m-0">
              <MainTab />
            </TabsContent>

            <TabsContent value="subscriptions" className="h-full m-0">
              <SubscriptionsTab />
            </TabsContent>

            <TabsContent value="settings" className="h-full m-0">
              <SettingsTab />
            </TabsContent>
          </div>
        </Tabs>
      </div>
    </div>
  );
}

export default App;
