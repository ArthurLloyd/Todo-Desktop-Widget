import { useI18n } from "../lib/i18n";

type Props = {
  title: string;
  body: string;
  confirmLabel: string;
  cancelLabel?: string;
  danger?: boolean;
  busy?: boolean;
  onCancel: () => void;
  onConfirm: () => void;
};

export function ConfirmDialog({
  title,
  body,
  confirmLabel,
  cancelLabel,
  danger = false,
  busy = false,
  onCancel,
  onConfirm,
}: Props) {
  const { t } = useI18n();
  return (
    <div className="confirm-dialog" role="alertdialog" aria-modal="true" aria-label={title}>
      <section className="confirm-dialog__card">
        <h3>{title}</h3>
        <p>{body}</p>
        <div className="confirm-dialog__actions">
          <button className="secondary-button" type="button" disabled={busy} onClick={onCancel}>
            {cancelLabel || t("common.cancel")}
          </button>
          <button className={danger ? "danger-button" : "text-button"} type="button" disabled={busy} onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </section>
    </div>
  );
}
