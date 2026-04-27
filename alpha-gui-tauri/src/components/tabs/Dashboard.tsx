import type { DeviceFile } from "../../api/types";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Icon } from "../ui/Icon";

interface Props {
  files: DeviceFile[];
  onBackupAll: () => void;
  onBackupFile: (slot: number) => void;
  onRefresh: () => void;
}

export function Dashboard({ files, onBackupAll, onBackupFile, onRefresh }: Props) {
  return (
    <div className="space-y-8">
      <div className="flex flex-col justify-between gap-4 md:flex-row md:items-end">
        <div>
          <h2 className="text-h1 text-on-surface">Dashboard</h2>
          <p className="mt-1 text-body-md text-on-surface-variant">
            Manage files on the connected AlphaSmart NEO.
          </p>
        </div>
        <Button icon={<Icon name="backup" className="text-lg" />} onClick={onBackupAll}>
          Backup All Files
        </Button>
      </div>
      <Card className="overflow-hidden">
        <div className="flex items-center justify-between border-b border-outline-variant bg-surface-container-low px-6 py-4">
          <h3 className="text-xs font-bold uppercase tracking-[0.18em] text-on-surface-variant">
            Device Files
          </h3>
          <span className="text-sm text-on-surface-variant">{files.length} files</span>
        </div>
        {files.length === 0 ? (
          <div className="flex items-center justify-between gap-4 p-6">
            <p className="text-on-surface-variant">No files listed yet.</p>
            <Button variant="secondary" onClick={onRefresh}>
              Refresh
            </Button>
          </div>
        ) : (
          <div className="divide-y divide-outline-variant/60">
            {files.map((file) => (
              <div key={file.slot} className="flex items-center justify-between gap-4 p-4 hover:bg-surface-container-low">
                <div className="flex items-center gap-4">
                  <Icon name="document" className="text-outline" />
                  <div>
                  <p className="text-lg font-medium">{file.name || `File ${file.slot}`}</p>
                  <p className="text-sm text-on-surface-variant">
                    Slot {file.slot} · {formatBytes(file.attributeBytes)}
                  </p>
                  </div>
                </div>
                <Button variant="secondary" onClick={() => onBackupFile(file.slot)}>
                  Backup
                </Button>
              </div>
            ))}
          </div>
        )}
      </Card>
    </div>
  );
}

export function formatBytes(bytes: number | null | undefined) {
  if (bytes == null) return "Unknown size";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
