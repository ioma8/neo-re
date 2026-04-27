import { Button } from "./Button";
import { Card } from "./Card";

export interface ConfirmRequest {
  title: string;
  message: string;
  confirmLabel: string;
  destructive?: boolean;
  onConfirm: () => void;
}

interface Props {
  request: ConfirmRequest | null;
  onCancel: () => void;
}

export function ConfirmDialog({ request, onCancel }: Props) {
  if (!request) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-slate-950/35 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-lg p-6">
        <p className="text-xs font-bold uppercase tracking-[0.18em] text-primary">
          Confirmation required
        </p>
        <h2 className="mt-2 text-2xl font-semibold">{request.title}</h2>
        <p className="mt-4 text-on-surface-variant">{request.message}</p>
        <div className="mt-7 flex justify-end gap-3">
          <Button variant="secondary" onClick={onCancel}>
            Cancel
          </Button>
          <Button
            variant={request.destructive ? "danger" : "primary"}
            onClick={() => {
              request.onConfirm();
              onCancel();
            }}
          >
            {request.confirmLabel}
          </Button>
        </div>
      </Card>
    </div>
  );
}
