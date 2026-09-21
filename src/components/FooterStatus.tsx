import clsx from "clsx";
import type { SyncStatus } from "../lib/types";
import { formatSyncTime, syncLabel } from "../lib/format";
import { useI18n } from "../lib/i18n";
import { classifySyncPresentation } from "../lib/widgetPresentation";

type Props = {
  inboxCount: number;
  syncStatus: SyncStatus;
  onRetry: () => void;
};

export function FooterStatus({ inboxCount, syncStatus, onRetry }: Props) {
  const { t } = useI18n();
  const presentation = classifySyncPresentation(syncStatus);
  const detail = presentation === "offline"
    ? t("sync.offlineHint")
    : presentation === "error"
      ? null
      : formatSyncTime(syncStatus.lastSyncedAt, t);

  return (
    <footer className="footer-status">
      <span>{t("footer.inbox", { count: inboxCount })}</span>
      {presentation === "error" ? (
        <button className="footer-status__retry" type="button" onClick={onRetry}>
          {t("sync.error")} · {t("common.retry")}
        </button>
      ) : (
        <span className={clsx("footer-status__sync", `footer-status__sync--${syncStatus.state}`)}>
          {syncLabel(syncStatus.state, t)}{detail ? ` · ${detail}` : ""}
        </span>
      )}
    </footer>
  );
}
