import { ArrowDown, ArrowUp, Trash2 } from "lucide-react";
import { useEffect, useState } from "react";
import { useI18n } from "../lib/i18n";
import type { TaskList, WidgetPane, WidgetPaneInput, WidgetTheme } from "../lib/types";

type Props = {
  pane: WidgetPane;
  index: number;
  total: number;
  taskLists: TaskList[];
  themes: WidgetTheme[];
  disabled: boolean;
  onUpdate: (input: WidgetPaneInput) => void;
  onMove: (direction: -1 | 1) => void;
  onDelete: () => void;
};

export function WidgetPaneEditor({
  pane,
  index,
  total,
  taskLists,
  themes,
  disabled,
  onUpdate,
  onMove,
  onDelete,
}: Props) {
  const { t } = useI18n();
  const [title, setTitle] = useState(pane.titleOverride ?? "");
  const [maxTasks, setMaxTasks] = useState(pane.maxTasks?.toString() ?? "");
  const list = taskLists.find((item) => item.id === pane.listId);
  const displayName = pane.titleOverride || list?.displayName ||
    (pane.listId ? t("pane.listUnavailable") : t("pane.selectList"));

  useEffect(() => setTitle(pane.titleOverride ?? ""), [pane.id, pane.titleOverride]);
  useEffect(() => setMaxTasks(pane.maxTasks?.toString() ?? ""), [pane.id, pane.maxTasks]);

  function buildInput(patch: Partial<WidgetPaneInput>): WidgetPaneInput {
    return {
      widgetInstanceId: pane.widgetInstanceId,
      listId: pane.listId ?? null,
      titleOverride: pane.titleOverride ?? null,
      themeId: pane.themeId ?? null,
      sortOrder: pane.sortOrder,
      showCompleted: pane.showCompleted ?? null,
      maxTasks: pane.maxTasks ?? null,
      sizeWeight: pane.sizeWeight,
      ...patch,
    };
  }

  function saveTitle() {
    const normalized = title.trim();
    if (normalized === (pane.titleOverride ?? "")) return;
    onUpdate(buildInput({ titleOverride: normalized || null }));
  }

  function saveMaxTasks() {
    const normalized = maxTasks.trim();
    const value = normalized ? Number(normalized) : null;
    if (value !== null && (!Number.isInteger(value) || value < 1 || value > 500)) {
      setMaxTasks(pane.maxTasks?.toString() ?? "");
      return;
    }
    if (value === (pane.maxTasks ?? null)) return;
    onUpdate(buildInput({ maxTasks: value }));
  }

  return (
    <article className="widget-pane-editor">
      <header className="widget-pane-editor__header">
        <strong>{displayName}</strong>
        <div className="widget-pane-editor__order">
          <button
            className="icon-button"
            type="button"
            disabled={disabled || index === 0}
            title={t("widgetManager.movePaneUp", { name: displayName })}
            aria-label={t("widgetManager.movePaneUp", { name: displayName })}
            onClick={() => onMove(-1)}
          >
            <ArrowUp size={14} />
          </button>
          <button
            className="icon-button"
            type="button"
            disabled={disabled || index === total - 1}
            title={t("widgetManager.movePaneDown", { name: displayName })}
            aria-label={t("widgetManager.movePaneDown", { name: displayName })}
            onClick={() => onMove(1)}
          >
            <ArrowDown size={14} />
          </button>
        </div>
      </header>

      <label>
        <span>{t("widgetManager.paneTitle")}</span>
        <input
          type="text"
          value={title}
          maxLength={120}
          disabled={disabled}
          placeholder={list?.displayName || t("pane.selectList")}
          onChange={(event) => setTitle(event.target.value)}
          onBlur={saveTitle}
          onKeyDown={(event) => {
            if (event.key === "Enter") event.currentTarget.blur();
          }}
        />
      </label>

      <label>
        <span>{t("widgetManager.paneList")}</span>
        <select
          value={list?.id || (pane.listId ? "__missing__" : "")}
          disabled={disabled}
          onChange={(event) => onUpdate(buildInput({ listId: event.target.value || null }))}
        >
          <option value="">{t("pane.selectList")}</option>
          {pane.listId && !list ? (
            <option value="__missing__" disabled>{t("pane.listUnavailable")}</option>
          ) : null}
          {taskLists.map((item) => (
            <option key={item.id} value={item.id}>{item.displayName}</option>
          ))}
        </select>
      </label>

      <label>
        <span>{t("theme.paneTheme")}</span>
        <select
          value={
            !pane.themeId || themes.some((theme) => theme.id === pane.themeId)
              ? pane.themeId || ""
              : "__missing__"
          }
          disabled={disabled}
          onChange={(event) => onUpdate(buildInput({ themeId: event.target.value || null }))}
        >
          <option value="">{t("theme.inheritWindow")}</option>
          {pane.themeId && !themes.some((theme) => theme.id === pane.themeId) ? (
            <option value="__missing__" disabled>{t("theme.unavailable")}</option>
          ) : null}
          {themes.map((theme) => (
            <option key={theme.id} value={theme.id}>{displayThemeName(theme, t)}</option>
          ))}
        </select>
      </label>

      <div className="widget-pane-editor__overrides">
        <label>
          <span>{t("settings.showCompleted")}</span>
          <select
            value={pane.showCompleted === null || pane.showCompleted === undefined ? "inherit" : String(pane.showCompleted)}
            disabled={disabled}
            onChange={(event) => onUpdate(buildInput({
              showCompleted: event.target.value === "inherit" ? null : event.target.value === "true",
            }))}
          >
            <option value="inherit">{t("widgetManager.inheritGlobal")}</option>
            <option value="true">{t("widgetManager.show")}</option>
            <option value="false">{t("widgetManager.hide")}</option>
          </select>
        </label>
        <label>
          <span>{t("widgetManager.maxTasks")}</span>
          <input
            type="number"
            min={1}
            max={500}
            value={maxTasks}
            disabled={disabled}
            placeholder={t("widgetManager.unlimited")}
            onChange={(event) => setMaxTasks(event.target.value)}
            onBlur={saveMaxTasks}
            onKeyDown={(event) => {
              if (event.key === "Enter") event.currentTarget.blur();
            }}
          />
        </label>
      </div>

      <button
        className="details-delete-button widget-pane-editor__delete"
        type="button"
        disabled={disabled || total <= 1}
        onClick={onDelete}
      >
        <Trash2 size={14} />
        {total <= 1 ? t("pane.atLeastOne") : t("widgetManager.deletePane")}
      </button>
    </article>
  );
}

function displayThemeName(theme: WidgetTheme, t: ReturnType<typeof useI18n>["t"]) {
  if (theme.id === "default") return t("theme.default");
  if (theme.id === "ocean") return t("theme.ocean");
  if (theme.id === "violet") return t("theme.violet");
  if (theme.id === "forest") return t("theme.forest");
  return theme.name;
}
