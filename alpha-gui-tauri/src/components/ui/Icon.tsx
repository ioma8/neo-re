import {
  mdiAlert,
  mdiCellphoneLink,
  mdiCheckCircleOutline,
  mdiChip,
  mdiContentSaveOutline,
  mdiFileDocumentOutline,
  mdiHelpCircleOutline,
  mdiInformationOutline,
  mdiMonitorDashboard,
  mdiPuzzleOutline,
  mdiUsbPort,
  mdiUploadOutline,
  mdiViewDashboardOutline,
} from "@mdi/js";
import MdiIcon from "@mdi/react";
import clsx from "clsx";

const icons = {
  about: mdiHelpCircleOutline,
  applets: mdiPuzzleOutline,
  backup: mdiContentSaveOutline,
  cable: mdiCellphoneLink,
  dashboard: mdiViewDashboardOutline,
  document: mdiFileDocumentOutline,
  firmware: mdiChip,
  info: mdiInformationOutline,
  os: mdiMonitorDashboard,
  upload: mdiUploadOutline,
  usb: mdiUsbPort,
  verified: mdiCheckCircleOutline,
  warning: mdiAlert,
} as const;

export type IconName = keyof typeof icons;

interface Props {
  name: IconName;
  className?: string;
  filled?: boolean;
}

export function Icon({ name, className, filled = false }: Props) {
  return (
    <span
      aria-hidden="true"
      className={clsx(
        "inline-flex size-6 shrink-0 items-center justify-center leading-none",
        filled && "stroke-[2.5]",
        className,
      )}
    >
      <MdiIcon path={icons[name]} size="1em" />
    </span>
  );
}
