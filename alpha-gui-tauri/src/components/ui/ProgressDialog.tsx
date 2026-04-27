import type { ProgressEvent } from "../../api/types";
import { Card } from "./Card";

interface Props {
  progress: ProgressEvent | null;
  error: string | null;
  onClose: () => void;
}

export function ProgressDialog({ progress, error, onClose }: Props) {
  if (!progress && !error) {
    return null;
  }
  const value =
    progress?.completed != null && progress.total ? progress.completed / progress.total : null;

  return (
    <div className="fixed inset-0 z-50 grid place-items-center bg-slate-950/35 p-6 backdrop-blur-sm">
      <Card className="w-full max-w-lg p-6">
        <div className="flex items-start justify-between gap-4">
          <div>
            <p className="text-xs font-bold uppercase tracking-[0.18em] text-primary">
              {error ? "Operation failed" : "Operation in progress"}
            </p>
            <h2 className="mt-2 text-2xl font-semibold">{error ? "Action needed" : progress?.title}</h2>
          </div>
          <button className="rounded-full px-3 py-1 text-on-surface-variant hover:bg-surface-container" onClick={onClose}>
            Close
          </button>
        </div>
        {error ? (
          <p className="mt-5 rounded-xl bg-error-container p-4 text-sm text-on-error-container">
            {error}
          </p>
        ) : (
          <>
            <p className="mt-5 text-on-surface-variant">
              {progress?.phase}
              {progress?.item ? `: ${progress.item}` : ""}
            </p>
            <div className="mt-5 h-3 overflow-hidden rounded-full bg-surface-container-high">
              <div
                className="h-full rounded-full bg-primary transition-all"
                style={{ width: value == null ? "45%" : `${Math.max(4, value * 100)}%` }}
              />
            </div>
            <p className="mt-3 text-sm text-on-surface-variant">
              {progress?.completed != null && progress.total != null
                ? `${progress.completed} / ${progress.total}`
                : "Working..."}
            </p>
          </>
        )}
      </Card>
    </div>
  );
}
