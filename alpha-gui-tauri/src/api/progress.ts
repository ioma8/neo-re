import { listen } from "@tauri-apps/api/event";
import type { ProgressEvent } from "./types";

export function listenForProgress(handler: (event: ProgressEvent) => void) {
  return listen<ProgressEvent>("alpha-progress", (event) => handler(event.payload));
}
