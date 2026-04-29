import { Button } from "./Button";

interface Props {
  open: boolean;
  log: string;
  onClose: () => void;
}

export function DiagnosticLogDialog({ open, log, onClose }: Props) {
  if (!open) return null;

  async function copyLog() {
    await navigator.clipboard.writeText(log);
  }

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-black/35 p-4">
      <div className="flex max-h-[85vh] w-full max-w-4xl flex-col rounded-xl border border-outline-variant bg-surface-container-lowest shadow-2xl">
        <div className="flex items-center justify-between border-b border-outline-variant p-5">
          <div>
            <p className="text-xs font-bold uppercase tracking-[0.18em] text-primary">
              Recovery Diagnostics
            </p>
            <h3 className="mt-1 text-2xl font-semibold text-on-surface">
              Diagnostic Log
            </h3>
          </div>
          <button className="text-on-surface-variant hover:text-on-surface" onClick={onClose}>
            Close
          </button>
        </div>
        <pre className="m-0 flex-1 overflow-auto whitespace-pre-wrap break-words bg-surface-container-low p-5 font-mono text-xs leading-5 text-on-surface">
          {log}
        </pre>
        <div className="flex justify-end gap-3 border-t border-outline-variant p-4">
          <Button variant="secondary" onClick={() => void copyLog()}>
            Copy Log
          </Button>
          <Button onClick={onClose}>Done</Button>
        </div>
      </div>
    </div>
  );
}
