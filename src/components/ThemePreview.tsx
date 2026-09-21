import { useMemo } from "react";
import { useI18n } from "../lib/i18n";
import { isThemeDraftValid } from "../lib/themeEditor";
import { themeToCssVariables } from "../lib/themeRuntime";
import type { WidgetThemeConfig } from "../lib/types";

type Props = {
  config: WidgetThemeConfig;
  fallbackConfig: WidgetThemeConfig;
  windowOpacity: number;
};

export function ThemePreview({ config, fallbackConfig, windowOpacity }: Props) {
  const { t } = useI18n();
  const valid = isThemeDraftValid({ name: "Preview", config });
  const themeStyle = useMemo(
    () => themeToCssVariables(valid ? config : fallbackConfig, windowOpacity),
    [config, fallbackConfig, valid, windowOpacity],
  );

  return (
    <section className="theme-preview-stage" aria-label={t("themeEditor.preview")}>
      <div className="theme-preview-window" style={themeStyle}>
        <header className="theme-preview-window__header">
          <strong>{t("themeEditor.previewWidget")}</strong>
          <span>{t("themeEditor.previewButton")}</span>
        </header>
        <div className="theme-preview-pane">
          <PreviewTask label={t("themeEditor.previewNormal")} />
          <PreviewTask label={t("themeEditor.previewToday")} state="today" meta={t("task.today")} />
          <PreviewTask label={t("themeEditor.previewOverdue")} state="overdue" meta="2026-01-01" />
          <PreviewTask label={t("themeEditor.previewImportant")} state="important" meta="★" />
          <PreviewTask label={t("themeEditor.previewPending")} state="pending" meta={t("task.pending")} />
          <PreviewTask label={t("themeEditor.previewCompleted")} state="completed" meta="✓" />
        </div>
        {!valid ? <span className="theme-preview-invalid">{t("themeEditor.previewInvalid")}</span> : null}
      </div>
    </section>
  );
}

function PreviewTask({
  label,
  state = "normal",
  meta,
}: {
  label: string;
  state?: "normal" | "today" | "overdue" | "important" | "pending" | "completed";
  meta?: string;
}) {
  return (
    <div className={`theme-preview-task theme-preview-task--${state}`}>
      <span className="theme-preview-task__checkbox">{state === "completed" ? "✓" : ""}</span>
      <span>{label}</span>
      {meta ? <small>{meta}</small> : null}
    </div>
  );
}
