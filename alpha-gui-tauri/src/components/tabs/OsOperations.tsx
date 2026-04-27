import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Icon } from "../ui/Icon";

interface Props {
  onBackupEverything: () => void;
  onFlashSystem: () => void;
}

export function OsOperations({ onBackupEverything, onFlashSystem }: Props) {
  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-h1 text-on-surface">OS Operations</h2>
        <p className="mt-1 text-body-lg text-on-surface-variant">
          Back up the device before firmware or system changes.
        </p>
      </div>
      <Card className="flex flex-col justify-between gap-5 p-6 md:flex-row md:items-center">
        <div>
          <h3 className="text-2xl font-semibold">System Backup</h3>
          <p className="mt-2 text-on-surface-variant">
            Create a complete snapshot of files and SmartApplets before any destructive operation.
          </p>
        </div>
        <Button
          icon={<Icon name="backup" className="text-lg" />}
          onClick={onBackupEverything}
        >
          Backup Everything
        </Button>
      </Card>
      <div className="grid gap-6 md:grid-cols-2">
        <Card className="overflow-hidden border-error-container opacity-75">
          <div className="flex items-center gap-3 border-b border-error-container bg-error-container/20 p-5">
            <Icon name="firmware" className="text-error" />
            <h3 className="text-2xl font-semibold">Reflash Firmware</h3>
          </div>
          <div className="space-y-5 p-6">
            <p className="text-on-surface-variant">
              Disabled: no separate validated firmware image is bundled yet. AlphaGUI only exposes proven operations.
            </p>
            <Button variant="secondary" disabled>
              Firmware Flash Unavailable
            </Button>
          </div>
        </Card>
        <Card className="overflow-hidden border-error-container">
          <div className="flex items-center gap-3 border-b border-error-container bg-error-container/30 p-5">
            <Icon name="os" className="text-error" />
            <h3 className="text-2xl font-semibold">Reflash System</h3>
          </div>
          <div className="space-y-5 p-6">
            <p className="text-on-surface-variant">
              Reinstall the bundled AlphaSmart NEO OS image. This can brick the device if interrupted.
            </p>
            <Button
              variant="danger"
              icon={<Icon name="warning" className="text-lg" />}
              onClick={onFlashSystem}
            >
              Reflash Bundled OS
            </Button>
          </div>
        </Card>
        <Card className="p-6">
          <h3 className="flex items-center gap-3 text-2xl font-semibold">
            <Icon name="verified" className="text-secondary" />
            Validated Small ROM Operations
          </h3>
          <p className="mt-2 text-on-surface-variant">
            No extra Small ROM operations are exposed yet because only validated workflows should be user-facing.
          </p>
        </Card>
      </div>
    </div>
  );
}
