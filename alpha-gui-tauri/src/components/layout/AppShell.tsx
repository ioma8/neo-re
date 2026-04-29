import clsx from "clsx";
import type { ReactNode } from "react";
import { Icon, type IconName } from "../ui/Icon";

export type TabKey = "dashboard" | "applets" | "os" | "about";

const tabs: Array<{ key: TabKey; label: string; icon: IconName }> = [
  { key: "dashboard", label: "Dashboard", icon: "dashboard" },
  { key: "applets", label: "SmartApplets", icon: "applets" },
  { key: "os", label: "OS Operations", icon: "os" },
  { key: "about", label: "About", icon: "about" },
];

interface Props {
  activeTab: TabKey;
  onTabChange: (tab: TabKey) => void;
  connectedLabel: string;
  children: ReactNode;
}

export function AppShell({ activeTab, onTabChange, connectedLabel, children }: Props) {
  return (
    <div className="mobile-screen bg-background text-on-surface">
      <aside className="fixed inset-y-0 left-0 z-40 hidden w-64 border-r border-gray-200 bg-gray-50 font-sans text-sm tracking-normal lg:flex lg:flex-col">
        <div className="px-6 py-8">
          <h1 className="text-xl font-black text-primary">AlphaGUI</h1>
          <p className="mt-1 text-sm text-on-surface-variant">Manager for Alpha writing devices</p>
        </div>
        <nav className="flex w-full flex-col gap-2 px-4">
          {tabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => onTabChange(tab.key)}
              className={clsx(
                "flex w-full items-center gap-3 rounded-lg px-3 py-2 text-left transition-colors",
                activeTab === tab.key
                  ? "border-r-4 border-primary bg-blue-50 font-semibold text-primary"
                  : "text-gray-600 hover:bg-gray-100",
              )}
            >
              <Icon name={tab.icon} filled={activeTab === tab.key} className="text-[28px]" />
              {tab.label}
            </button>
          ))}
        </nav>
        <div className="absolute bottom-6 left-6 rounded-full bg-surface-container-high px-4 py-3 text-on-surface-variant">
          {connectedLabel}
        </div>
      </aside>
      <main className="min-h-screen safe-main-bottom lg:ml-64 lg:pb-0">
        <header className="safe-top sticky top-0 z-20 border-b border-gray-200 bg-white px-4 py-2 font-sans text-sm font-medium antialiased lg:hidden">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Icon name="usb" className="text-primary" />
              <p className="text-lg font-bold tracking-tight text-primary">AlphaGUI</p>
            </div>
            <div className="text-right">
              <p className="text-sm text-on-surface-variant">{connectedLabel}</p>
            </div>
          </div>
        </header>
        <div className="mx-auto max-w-[1440px] p-md lg:p-margin">{children}</div>
      </main>
      <nav className="mobile-bottom-nav fixed inset-x-0 bottom-0 z-30 grid grid-cols-4 border-t border-gray-200 bg-white text-[10px] font-medium uppercase tracking-wider lg:hidden">
        {tabs.map((tab) => (
          <button
            key={tab.key}
            onClick={() => onTabChange(tab.key)}
            className={clsx(
              "flex flex-col items-center justify-center rounded-lg p-1 transition-transform active:scale-95",
              activeTab === tab.key ? "bg-blue-50/50 font-bold text-primary" : "text-gray-500",
            )}
          >
            <Icon name={tab.icon} filled={activeTab === tab.key} className="text-[30px]" />
            {tab.label}
          </button>
        ))}
      </nav>
    </div>
  );
}
