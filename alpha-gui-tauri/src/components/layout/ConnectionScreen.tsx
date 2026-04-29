import type { DeviceMode } from "../../api/types";
import { Button } from "../ui/Button";
import { Card } from "../ui/Card";
import { Icon } from "../ui/Icon";

interface Props {
  mode: DeviceMode;
  scanning: boolean;
  message: string;
  showDebugOpen: boolean;
  onDebugOpen: () => void;
}

export function ConnectionScreen({
  mode,
  scanning,
  message,
  showDebugOpen,
  onDebugOpen,
}: Props) {
  return (
    <main className="mobile-screen mobile-page-padding relative overflow-hidden bg-[radial-gradient(circle_at_30%_15%,#eaf1ff_0,#faf8ff_36%,#f7f9ff_100%)] text-on-surface md:p-margin">
      <div className="pointer-events-none absolute left-[-8rem] top-[-10rem] size-[24rem] rounded-full bg-primary/10 blur-3xl" />
      <div className="pointer-events-none absolute bottom-[-12rem] right-[-10rem] size-[28rem] rounded-full bg-secondary-container/70 blur-3xl" />
      <div className="connection-center mx-auto flex max-w-6xl items-center justify-center">
        <Card className="relative flex w-full max-w-xl flex-col items-center gap-lg overflow-hidden p-xl text-center shadow-[0_24px_80px_rgba(0,40,120,0.14)]">
          <div className="absolute inset-x-0 top-0 h-1 bg-gradient-to-r from-primary via-inverse-primary to-secondary-container" />
          <div className="flex items-center gap-3 self-start text-left">
            <div className="grid size-11 place-items-center rounded-xl bg-primary text-on-primary shadow-sm">
              <Icon name="usb" className="text-2xl" />
            </div>
            <div>
              <p className="text-h3 text-primary">AlphaGUI</p>
              <p className="text-body-sm text-on-surface-variant">AlphaSmart devices manager</p>
            </div>
          </div>
          <div className="relative grid size-[128px] place-items-center rounded-[2rem] border border-primary-fixed bg-gradient-to-br from-primary-fixed/80 to-surface-container-low shadow-inner">
            <div className="absolute inset-3 rounded-[1.5rem] border-2 border-dashed border-white/70" />
            <Icon name="cable" className="relative text-[58px] text-primary" />
          </div>
          <div className="space-y-sm">
            <h1 className="text-h2 text-on-surface">Awaiting Device</h1>
            <p className="text-body-lg text-on-surface-variant">
              Connect your AlphaSmart NEO over USB. AlphaGUI will detect it and switch HID mode automatically.
            </p>
          </div>
          <div className="w-full overflow-hidden rounded-xl border border-outline-variant bg-surface-container-lowest text-left shadow-sm">
            <div className="flex items-center gap-3 px-5 py-4">
              <span className="relative flex size-3 shrink-0">
                <span className="absolute inline-flex size-full animate-ping rounded-full bg-primary opacity-40" />
                <span className="relative inline-flex size-3 rounded-full bg-primary" />
              </span>
              <div>
                <p className="text-sm font-semibold uppercase tracking-[0.14em] text-primary">
                  Connection
                </p>
                <p className="mt-1 text-body-md text-on-surface">{message}</p>
              </div>
            </div>
            <div className="h-1.5 overflow-hidden bg-surface-container-high">
              <div className="h-full w-1/2 rounded-r-full bg-gradient-to-r from-primary to-inverse-primary connection-progress" />
            </div>
          </div>
          {mode === "hid" && (
            <div className="rounded-lg border border-secondary-container bg-secondary-container/70 p-4 text-left text-sm text-on-secondary-container">
              HID keyboard mode detected. AlphaGUI is switching the device to direct USB mode automatically.
            </div>
          )}
          <div className="flex w-full gap-3 rounded-xl border border-outline-variant bg-secondary-container/80 p-md text-left text-body-sm text-on-secondary-container">
            <Icon name="info" className="text-tertiary" />
            <p>
              On phones, install and run the Alpha USB SmartApplet from this desktop app first, then connect the device to the phone.
            </p>
          </div>
          {showDebugOpen && (
            <Button variant="ghost" className="mt-5" onClick={onDebugOpen}>
              Debug: Open UI Without Device
            </Button>
          )}
        </Card>
      </div>
    </main>
  );
}
