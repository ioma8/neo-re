import { Card } from "../ui/Card";
import { Icon } from "../ui/Icon";

export function About() {
  return (
    <div className="max-w-3xl space-y-6">
      <Card className="p-8">
        <div className="flex items-start justify-between gap-5">
          <div>
            <div className="flex items-center gap-4">
              <div className="grid size-12 place-items-center rounded-lg border border-outline-variant bg-surface-container">
                <Icon name="usb" className="text-3xl text-primary" />
              </div>
              <h2 className="text-h1 text-on-surface">AlphaGUI</h2>
            </div>
            <p className="mt-5 text-lg text-on-surface-variant">
              Manager for Alpha writing devices. AlphaGUI manages files, SmartApplets,
              flashing, backups, and validated recovery operations.
            </p>
          </div>
          <span className="rounded-full bg-surface-container px-4 py-2 font-mono text-sm">
            v0.1.0
          </span>
        </div>
        <div className="mt-8 flex flex-wrap gap-4">
          <a className="font-semibold text-primary" href="https://github.com/ioma8/neo-re" target="_blank" rel="noreferrer">
            GitHub Repository
          </a>
          <a className="font-semibold text-primary" href="https://github.com/ioma8/neo-re/tree/master/docs" target="_blank" rel="noreferrer">
            Documentation
          </a>
        </div>
      </Card>
      <Card className="flex items-start gap-3 border-error/30 bg-error-container p-5 text-on-error-container">
        <Icon name="warning" className="text-error" />
        <div>
          <p className="font-semibold">Critical Warning</p>
          <p className="mt-2">
            Use at your own risk. Flashing firmware, system images, or applet areas can brick the device if interrupted or used incorrectly.
          </p>
        </div>
      </Card>
    </div>
  );
}
