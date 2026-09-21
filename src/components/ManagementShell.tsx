import { ArrowLeft, X } from "lucide-react";
import type { PointerEvent, ReactNode } from "react";
import { api } from "../lib/api";
import { useI18n } from "../lib/i18n";
import type { ManagementThemeCssVariables } from "../lib/managementTheme";

type Props = {
  title: string;
  hint?: string;
  themeStyle: ManagementThemeCssVariables;
  onBack?: () => void;
  onClose: () => void;
  children: ReactNode;
};

export function ManagementShell({ title, hint, themeStyle, onBack, onClose, children }: Props) {
  const { t } = useI18n();

  function handlePointerDown(event: PointerEvent<HTMLElement>) {
    if (event.button !== 0) return;
    const target = event.target as HTMLElement;
    if (target.closest("button, input, select, textarea, a, label")) return;
    void api.startWindowDrag();
  }

  return (
    <section className="management-shell" aria-label={title} style={themeStyle}>
      <header
        className="management-shell__header"
        data-tauri-drag-region
        onPointerDown={handlePointerDown}
      >
        <div className="management-shell__navigation">
          {onBack ? (
            <button
              className="icon-button"
              type="button"
              title={t("common.back")}
              aria-label={t("common.back")}
              onClick={onBack}
            >
              <ArrowLeft size={16} />
            </button>
          ) : null}
          <div data-tauri-drag-region>
            <strong data-tauri-drag-region>{title}</strong>
            {hint ? <span data-tauri-drag-region>{hint}</span> : null}
          </div>
        </div>
        <button
          className="icon-button"
          type="button"
          title={t("common.close")}
          aria-label={t("common.close")}
          onClick={onClose}
        >
          <X size={16} />
        </button>
      </header>
      <div className="management-shell__body">{children}</div>
    </section>
  );
}
