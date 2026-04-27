import type { AppletChecklistRow } from "../../api/types";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Icon } from "../ui/Icon";
import { formatBytes } from "./Dashboard";

interface Props {
  rows: AppletChecklistRow[];
  checkedKeys: Set<string>;
  onToggle: (key: string) => void;
  onInstallAlphaUsb: () => void;
  onAddFromFile: () => void;
  onFlash: () => void;
  dirty: boolean;
}

export function SmartApplets({
  rows,
  checkedKeys,
  onToggle,
  onInstallAlphaUsb,
  onAddFromFile,
  onFlash,
  dirty,
}: Props) {
  const installedCount = rows.filter((row) => row.installed).length;

  return (
    <div className="space-y-8">
      <div className="flex flex-col justify-between gap-4 xl:flex-row xl:items-end">
        <div>
          <h2 className="text-h1 text-on-surface">SmartApplets</h2>
          <p className="mt-1 text-body-lg text-on-surface-variant">
            Manage bundled and installed SmartApplets.
          </p>
        </div>
        <Button
          className="xl:min-w-[28rem]"
          icon={<Icon name="cable" className="text-lg" />}
          onClick={onInstallAlphaUsb}
        >
          Flash Alpha USB SmartApplet for smartphone connection
        </Button>
      </div>
      <Card className="overflow-hidden">
        <div className="flex items-center justify-between border-b border-outline-variant bg-surface-container-low px-6 py-4">
          <h3 className="text-xl font-semibold">Available Applets</h3>
          <span className="text-sm font-bold uppercase tracking-[0.18em] text-on-surface-variant">
            {installedCount} installed
          </span>
        </div>
        <div className="divide-y divide-outline-variant/60">
          {rows.map((row) => (
            <label key={row.key} className="flex cursor-pointer items-start gap-4 p-4 hover:bg-surface-container-low">
              <input
                className="mt-1 size-5 accent-primary"
                type="checkbox"
                checked={checkedKeys.has(row.key)}
                onChange={() => onToggle(row.key)}
              />
              <div className="min-w-0 flex-1">
                <div className="flex flex-wrap items-center gap-2">
                  <p className="text-lg font-medium">{row.displayName}</p>
                  {row.version && (
                    <span className="rounded-full bg-surface-container-high px-2 py-1 text-xs text-on-surface-variant">
                      {row.version}
                    </span>
                  )}
                  {row.installed && (
                    <span className="rounded-full bg-secondary-container px-2 py-1 text-xs text-on-secondary-container">
                      Installed
                    </span>
                  )}
                </div>
                <p className="mt-1 text-sm text-on-surface-variant">
                  {row.sourceKind} · {formatBytes(row.size)}
                </p>
              </div>
            </label>
          ))}
        </div>
      </Card>
      <div className="flex flex-col justify-between gap-4 border-t border-outline-variant pt-6 sm:flex-row sm:items-center">
        <Button
          variant="ghost"
          icon={<Icon name="upload" className="text-lg" />}
          onClick={onAddFromFile}
        >
          Add new applet from file
        </Button>
        <Button disabled={!dirty} onClick={onFlash}>
          Flash to Device
        </Button>
      </div>
    </div>
  );
}
