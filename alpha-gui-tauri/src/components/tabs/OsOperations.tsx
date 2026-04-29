import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Icon } from "../ui/Icon";

interface Props {
  onBackupEverything: () => void;
  onFlashSystem: () => void;
  onFlashSystemFromSmallRom: () => void;
  onRestartDevice: () => void;
  onReadDiagnostics: () => void;
  onRestoreStockApplets: () => void;
}

export function OsOperations({
  onBackupEverything,
  onFlashSystem,
  onFlashSystemFromSmallRom,
  onRestartDevice,
  onReadDiagnostics,
  onRestoreStockApplets,
}: Props) {
  return (
    <div className="space-y-8">
      <div>
        <h2 className="text-h1 text-on-surface">OS Operations</h2>
        <p className="mt-1 text-body-lg text-on-surface-variant">
          Back up the device before system changes or recovery work.
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
      <div className="grid gap-6 lg:grid-cols-2">
        <Card className="overflow-hidden border-error-container">
          <div className="flex items-center gap-3 border-b border-error-container bg-error-container/30 p-5">
            <Icon name="os" className="text-error" />
            <h3 className="text-2xl font-semibold">Reflash Bundled OS</h3>
          </div>
          <div className="space-y-5 p-6">
            <p className="text-on-surface-variant">
              Reinstall the bundled stock OS image. This can brick the device if interrupted.
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
        <Card className="overflow-hidden">
          <div className="flex items-center gap-3 border-b border-outline-variant bg-surface-container-low p-5">
            <Icon name="verified" className="text-secondary" />
            <h3 className="text-2xl font-semibold">Restart Device</h3>
          </div>
          <div className="space-y-5 p-6">
            <p className="text-on-surface-variant">
              Restart the connected device. USB will disconnect while it reboots.
            </p>
            <Button variant="secondary" onClick={onRestartDevice}>
              Restart Device
            </Button>
          </div>
        </Card>
      </div>
      <section className="rounded-xl border border-error/30 bg-error-container/20 p-6">
        <div className="mb-5">
          <p className="text-xs font-bold uppercase tracking-[0.18em] text-error">
            Advanced Recovery
          </p>
          <h3 className="mt-1 text-2xl font-semibold">Validated repair tools</h3>
          <p className="mt-2 text-on-surface-variant">
            Use these only with a current backup. They are for damaged applet/file catalogs or diagnostics before repair.
          </p>
        </div>
        <div className="grid gap-5 lg:grid-cols-3">
          <Card className="flex flex-col justify-between gap-5 p-5">
            <div>
              <h4 className="text-xl font-semibold">Read Diagnostics</h4>
              <p className="mt-2 text-sm text-on-surface-variant">
                Read raw SmartApplet records and AlphaWord file attributes into a copyable technical log.
              </p>
            </div>
            <Button variant="secondary" onClick={onReadDiagnostics}>
              Read Diagnostics
            </Button>
          </Card>
          <Card className="flex flex-col justify-between gap-5 border-error-container p-5">
            <div>
              <h4 className="text-xl font-semibold">Restore Original Stock Applets</h4>
              <p className="mt-2 text-sm text-on-surface-variant">
                Clear the SmartApplet area, then install only bundled original stock applets one by one with verification.
              </p>
            </div>
            <Button variant="danger" onClick={onRestoreStockApplets}>
              Restore Stock Applets
            </Button>
          </Card>
          <Card className="flex flex-col justify-between gap-5 p-5">
            <div>
              <h4 className="text-xl font-semibold">Small ROM Recovery</h4>
              <p className="mt-2 text-sm text-on-surface-variant">
                Hold Right Shift + comma + period + slash while powering on, then enter the password &quot;ernie&quot;
                when prompted. Connect USB after the Small ROM Updater appears. SmartApplet operations are not
                available in Small ROM; use this only to reflash the bundled OS.
              </p>
            </div>
            <Button
              variant="danger"
              icon={<Icon name="warning" className="text-lg" />}
              onClick={onFlashSystemFromSmallRom}
            >
              Reflash OS from Small ROM
            </Button>
          </Card>
        </div>
      </section>
    </div>
  );
}
