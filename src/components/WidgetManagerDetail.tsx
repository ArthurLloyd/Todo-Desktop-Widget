import { ArrowLeft, Copy, ExternalLink, Palette, Plus, RotateCcw, Save, Trash2, X } from "lucide-react";
import { useEffect, useState } from "react";
import { useI18n } from "../lib/i18n";
import { InlineError } from "./InlineError";
import type {
  TaskList,
  WidgetContext,
  WidgetInstanceInput,
  WidgetLayoutMode,
  WidgetPane,
  WidgetPaneInput,
  WidgetTheme,
} from "../lib/types";
import { WidgetPaneEditor } from "./WidgetPaneEditor";

type Props = {
  context: WidgetContext;
  currentWidgetId: string | null;
  taskLists: TaskList[];
  themes: WidgetTheme[];
  busy: boolean;
  error?: string | null;
  onBack: () => void;
  onShow: () => void;
  onCloseWindow: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
  onSetPinned: (pinned: boolean) => void;
  onUpdateWidget: (input: WidgetInstanceInput) => void;
  onAddPane: () => void;
  onUpdatePane: (pane: WidgetPane, input: WidgetPaneInput) => void;
  onDeletePane: (pane: WidgetPane) => void;
  onMovePane: (pane: WidgetPane, direction: -1 | 1) => void;
  onResetPaneSizes: () => void;
  onManageThemes: () => void;
};

export function WidgetManagerDetail({
  context,
  currentWidgetId,
  taskLists,
  themes,
  busy,
  error,
  onBack,
  onShow,
  onCloseWindow,
  onDuplicate,
  onDelete,
  onSetPinned,
  onUpdateWidget,
  onAddPane,
  onUpdatePane,
  onDeletePane,
  onMovePane,
  onResetPaneSizes,
  onManageThemes,
}: Props) {
  const { t } = useI18n();
  const { instance, panes } = context;
  const [name, setName] = useState(instance.name);

  useEffect(() => setName(instance.name), [instance.id, instance.name]);

  function widgetInput(patch: Partial<WidgetInstanceInput>): WidgetInstanceInput {
    return {
      name: instance.name,
      layoutMode: instance.layoutMode,
      defaultThemeId: instance.defaultThemeId ?? null,
      ...patch,
    };
  }

  function saveName() {
    const normalized = name.trim();
    if (!normalized || normalized === instance.name) {
      setName(instance.name);
      return;
    }
    onUpdateWidget(widgetInput({ name: normalized }));
  }

  return (
    <section className="widget-manager-detail">
      <header className="widget-manager-detail__header">
        <button
          className="icon-button widget-manager-detail__back"
          type="button"
          title={t("widgetManager.backToList")}
          aria-label={t("widgetManager.backToList")}
          onClick={onBack}
        >
          <ArrowLeft size={16} />
        </button>
        <div>
          <strong>{instance.name}</strong>
          <span>
            {instance.isPrimary ? t("widget.primary") : t("widgetManager.secondary")}
            {instance.id === currentWidgetId ? ` · ${t("widget.current")}` : ""}
          </span>
        </div>
      </header>

      <div className="widget-manager-detail__status">
        <span data-open={instance.isOpen ? "true" : undefined} />
        {instance.isOpen ? t("widgetManager.openState") : t("widgetManager.closedState")}
        <span>·</span>
        {instance.pinned ? t("widgetManager.pinnedState") : t("widgetManager.normalState")}
      </div>

      <div className="widget-manager-detail__fields">
        <label className="widget-manager-field widget-manager-field--name">
          <span>{t("pane.windowName")}</span>
          <div>
            <input
              type="text"
              value={name}
              maxLength={64}
              disabled={busy}
              onChange={(event) => setName(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") saveName();
              }}
            />
            <button
              className="icon-button"
              type="button"
              disabled={busy || !name.trim() || name.trim() === instance.name}
              title={t("widgetManager.saveName")}
              aria-label={t("widgetManager.saveName")}
              onClick={saveName}
            >
              <Save size={14} />
            </button>
          </div>
        </label>

        <label className="widget-manager-field widget-manager-field--toggle">
          <span>{t("widgetManager.pinToDesktop")}</span>
          <input
            type="checkbox"
            checked={instance.pinned}
            disabled={busy}
            onChange={(event) => onSetPinned(event.target.checked)}
          />
        </label>

        <label className="widget-manager-field">
          <span>{t("pane.layout")}</span>
          <select
            value={instance.layoutMode}
            disabled={busy}
            onChange={(event) => onUpdateWidget(widgetInput({
              layoutMode: event.target.value as WidgetLayoutMode,
            }))}
          >
            <option value="single">{t("pane.layoutSingle")}</option>
            <option value="columns">{t("pane.layoutColumns")}</option>
            <option value="rows">{t("pane.layoutRows")}</option>
            <option value="grid">{t("pane.layoutGrid")}</option>
          </select>
        </label>

        <label className="widget-manager-field">
          <span>{t("theme.windowTheme")}</span>
          <select
            value={
              !instance.defaultThemeId || themes.some((theme) => theme.id === instance.defaultThemeId)
                ? instance.defaultThemeId || "default"
                : "__missing__"
            }
            disabled={busy}
            onChange={(event) => onUpdateWidget(widgetInput({ defaultThemeId: event.target.value }))}
          >
            {instance.defaultThemeId && !themes.some((theme) => theme.id === instance.defaultThemeId) ? (
              <option value="__missing__" disabled>{t("theme.unavailable")}</option>
            ) : null}
            {themes.map((theme) => (
              <option key={theme.id} value={theme.id}>{displayThemeName(theme, t)}</option>
            ))}
          </select>
        </label>
      </div>

      <div className="widget-manager-detail__theme-action">
        <button className="secondary-button" type="button" disabled={busy} onClick={onManageThemes}>
          <Palette size={15} />
          {t("themeEditor.manageThemes")}
        </button>
      </div>

      <div className="widget-manager-detail__pane-heading">
        <strong>{t("pane.sections")}</strong>
        {panes.length >= 2 ? (
          <button className="text-button" type="button" disabled={busy} onClick={onResetPaneSizes}>
            <RotateCcw size={14} />
            {t("widgetManager.resetPaneSizes")}
          </button>
        ) : null}
        <button className="text-button" type="button" disabled={busy} onClick={onAddPane}>
          <Plus size={15} />
          {t("pane.add")}
        </button>
      </div>

      <div className="widget-manager-detail__panes">
        {panes.map((pane, index) => (
          <WidgetPaneEditor
            key={pane.id}
            pane={pane}
            index={index}
            total={panes.length}
            taskLists={taskLists}
            themes={themes}
            disabled={busy}
            onUpdate={(input) => onUpdatePane(pane, input)}
            onMove={(direction) => onMovePane(pane, direction)}
            onDelete={() => onDeletePane(pane)}
          />
        ))}
      </div>

      {error ? <InlineError message={error} /> : null}

      <footer className="widget-manager-detail__actions">
        <button className="text-button" type="button" disabled={busy} onClick={onShow}>
          <ExternalLink size={15} />
          {instance.isOpen ? t("widget.show") : t("widget.open")}
        </button>
        {!instance.isPrimary && instance.isOpen ? (
          <button className="secondary-button" type="button" disabled={busy} onClick={onCloseWindow}>
            <X size={15} />
            {t("widgetManager.closeWindow")}
          </button>
        ) : null}
        <button className="secondary-button" type="button" disabled={busy} onClick={onDuplicate}>
          <Copy size={15} />
          {t("widgetManager.duplicate")}
        </button>
        {!instance.isPrimary ? (
          <button className="details-delete-button" type="button" disabled={busy} onClick={onDelete}>
            <Trash2 size={15} />
            {t("widgetManager.deleteWidget")}
          </button>
        ) : null}
      </footer>
    </section>
  );
}

function displayThemeName(theme: WidgetTheme, t: ReturnType<typeof useI18n>["t"]) {
  if (theme.id === "default") return t("theme.default");
  if (theme.id === "ocean") return t("theme.ocean");
  if (theme.id === "violet") return t("theme.violet");
  if (theme.id === "forest") return t("theme.forest");
  return theme.name;
}
