import { Copy, ExternalLink, Plus, Settings2, Trash2 } from "lucide-react";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { api } from "../lib/api";
import { localizeBackendMessage, useI18n } from "../lib/i18n";
import type { ManagementThemeCssVariables } from "../lib/managementTheme";
import type {
  TaskList,
  WidgetConfigChanged,
  WidgetContext,
  WidgetInstance,
  WidgetInstanceInput,
  WidgetPane,
  WidgetPaneInput,
  WidgetTheme,
} from "../lib/types";
import { WidgetManagerDetail } from "./WidgetManagerDetail";
import { ManagementShell } from "./ManagementShell";
import { ConfirmDialog } from "./ConfirmDialog";
import { InlineError } from "./InlineError";

type Props = {
  managementThemeStyle: ManagementThemeCssVariables;
  instances: WidgetInstance[];
  currentWidgetId: string | null;
  taskLists: TaskList[];
  themes: WidgetTheme[];
  onBack: () => void;
  onClose: () => void;
  onRefreshInstances: () => Promise<WidgetInstance[]>;
  onManageThemes: () => void;
};

export function WidgetManager({
  managementThemeStyle,
  instances,
  currentWidgetId,
  taskLists,
  themes,
  onBack,
  onClose,
  onRefreshInstances,
  onManageThemes,
}: Props) {
  const { t } = useI18n();
  const [selectedId, setSelectedId] = useState<string | null>(currentWidgetId || instances[0]?.id || null);
  const [context, setContext] = useState<WidgetContext | null>(null);
  const [view, setView] = useState<"list" | "detail">("list");
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const [error, setError] = useState<string | null>(null);
  const [createOpen, setCreateOpen] = useState(false);
  const [createListId, setCreateListId] = useState("");
  const [deleteTarget, setDeleteTarget] = useState<WidgetInstance | null>(null);

  const selectedInstance = instances.find((instance) => instance.id === selectedId) || null;

  useEffect(() => {
    if (selectedId && !instances.some((instance) => instance.id === selectedId)) {
      setSelectedId(currentWidgetId || instances[0]?.id || null);
      setView("list");
    }
  }, [currentWidgetId, instances, selectedId]);

  useEffect(() => {
    if (!selectedId) {
      setContext(null);
      return;
    }
    let cancelled = false;
    void api.getWidgetContext(selectedId).then((next) => {
      if (!cancelled) setContext(next);
    }).catch((loadError) => {
      if (!cancelled) setError(errorMessage(loadError, t));
    });
    return () => {
      cancelled = true;
    };
  }, [selectedId, selectedInstance?.updatedAt, t]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let disposed = false;
    void listen<WidgetConfigChanged>("widget-config-changed", (event) => {
      void onRefreshInstances().then(async (nextInstances) => {
        if (disposed) return;
        if (event.payload.widgetInstanceId !== selectedId) return;
        if (!selectedId || !nextInstances.some((instance) => instance.id === selectedId)) {
          setContext(null);
          return;
        }
        const nextContext = await api.getWidgetContext(selectedId);
        if (!disposed) setContext(nextContext);
      }).catch((refreshError) => {
        if (!disposed) setError(errorMessage(refreshError, t));
      });
    }).then((nextUnlisten) => {
      if (disposed) nextUnlisten();
      else unlisten = nextUnlisten;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [onRefreshInstances, selectedId, t]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent) {
      if (event.key !== "Escape") return;
      event.preventDefault();
      if (deleteTarget) setDeleteTarget(null);
      else onBack();
    }
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [deleteTarget, onBack]);

  async function refreshDetail(instanceId = selectedId) {
    const detailPromise = instanceId
      ? api.getWidgetContext(instanceId)
      : Promise.resolve<WidgetContext | null>(null);
    const [nextInstances, nextContext] = await Promise.all([
      onRefreshInstances(),
      detailPromise,
    ]);
    if (nextContext) setContext(nextContext);
    if (instanceId && !nextInstances.some((instance) => instance.id === instanceId)) {
      setSelectedId(currentWidgetId || nextInstances[0]?.id || null);
      setView("list");
    }
    return nextInstances;
  }

  async function runAction(action: () => Promise<void>) {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (actionError) {
      setError(errorMessage(actionError, t));
    } finally {
      busyRef.current = false;
      setBusy(false);
    }
  }

  function selectWidget(instanceId: string) {
    setSelectedId(instanceId);
    setView("detail");
    setError(null);
  }

  function createWidget(listId: string | null) {
    void runAction(async () => {
      const created = await api.createWidgetWindow(listId);
      await refreshDetail(created.id);
      setSelectedId(created.id);
      setCreateOpen(false);
      setCreateListId("");
      setView("detail");
    });
  }

  function updateWidget(input: WidgetInstanceInput) {
    if (!selectedId) return;
    void runAction(async () => {
      await api.updateManagedWidget(selectedId, input);
      await refreshDetail(selectedId);
    });
  }

  function updatePane(pane: WidgetPane, input: WidgetPaneInput) {
    if (!selectedId) return;
    void runAction(async () => {
      await api.updateManagedWidgetPane(selectedId, pane.id, input);
      await refreshDetail(selectedId);
    });
  }

  function movePane(pane: WidgetPane, direction: -1 | 1) {
    if (!selectedId || !context) return;
    const sourceIndex = context.panes.findIndex((item) => item.id === pane.id);
    const destinationIndex = sourceIndex + direction;
    if (sourceIndex < 0 || destinationIndex < 0 || destinationIndex >= context.panes.length) return;
    const orderedIds = context.panes.map((item) => item.id);
    [orderedIds[sourceIndex], orderedIds[destinationIndex]] = [
      orderedIds[destinationIndex],
      orderedIds[sourceIndex],
    ];
    void runAction(async () => {
      await api.reorderManagedWidgetPanes(selectedId, orderedIds);
      await refreshDetail(selectedId);
    });
  }

  function duplicateWidget(instanceId: string) {
    void runAction(async () => {
      const duplicated = await api.duplicateWidgetWindow(instanceId);
      await refreshDetail(duplicated.id);
      setSelectedId(duplicated.id);
      setView("detail");
    });
  }

  function deleteWidget() {
    const target = deleteTarget;
    if (!target) return;
    void runAction(async () => {
      await api.deleteManagedWidget(target.id);
      setDeleteTarget(null);
      const nextInstances = await onRefreshInstances();
      setContext(null);
      setSelectedId(currentWidgetId === target.id ? nextInstances[0]?.id || null : currentWidgetId || nextInstances[0]?.id || null);
      setView("list");
    });
  }

  return (
    <ManagementShell
      title={t("widgetManager.managerTitle")}
      hint={t("widgetManager.managerHint")}
      themeStyle={managementThemeStyle}
      onBack={onBack}
      onClose={onClose}
    >
      <section className="widget-manager-screen">
        <div className="widget-manager-screen__body" data-view={view}>
        <aside className="widget-manager-master">
          <div className="widget-manager-master__create">
            <button className="text-button" type="button" disabled={busy} onClick={() => setCreateOpen((open) => !open)}>
              <Plus size={15} />
              {t("widgetManager.newWidget")}
            </button>
            {createOpen ? (
              <div className="widget-manager-create-menu">
                <button className="secondary-button" type="button" disabled={busy} onClick={() => createWidget(null)}>
                  {t("widgetManager.createBlank")}
                </button>
                <label>
                  <span>{t("widgetManager.createFromList")}</span>
                  <select value={createListId} disabled={busy} onChange={(event) => setCreateListId(event.target.value)}>
                    <option value="">{t("widgetManager.chooseList")}</option>
                    {taskLists.map((list) => (
                      <option key={list.id} value={list.id}>{list.displayName}</option>
                    ))}
                  </select>
                </label>
                <button
                  className="secondary-button"
                  type="button"
                  disabled={busy || !createListId}
                  onClick={() => createWidget(createListId)}
                >
                  {t("widgetManager.create")}
                </button>
              </div>
            ) : null}
          </div>

          <div className="widget-manager-cards">
            {instances.map((instance) => (
              <WidgetManagerCard
                key={instance.id}
                instance={instance}
                current={instance.id === currentWidgetId}
                selected={instance.id === selectedId}
                themes={themes}
                busy={busy}
                onManage={() => selectWidget(instance.id)}
                onShow={() => void runAction(async () => {
                  await api.showManagedWidgetWindow(instance.id);
                  await refreshDetail(selectedId);
                })}
                onClose={() => void runAction(async () => {
                  await api.closeManagedWidgetWindow(instance.id);
                  await refreshDetail(selectedId);
                })}
                onDuplicate={() => duplicateWidget(instance.id)}
                onDelete={() => setDeleteTarget(instance)}
              />
            ))}
          </div>
          {error && view === "list" ? <InlineError message={error} /> : null}
        </aside>

        <main className="widget-manager-detail-column">
          {context && context.instance.id === selectedId ? (
            <WidgetManagerDetail
              context={context}
              currentWidgetId={currentWidgetId}
              taskLists={taskLists}
              themes={themes}
              busy={busy}
              error={error}
              onBack={() => setView("list")}
              onShow={() => void runAction(async () => {
                await api.showManagedWidgetWindow(context.instance.id);
                await refreshDetail(context.instance.id);
              })}
              onCloseWindow={() => void runAction(async () => {
                await api.closeManagedWidgetWindow(context.instance.id);
                await refreshDetail(context.instance.id);
              })}
              onDuplicate={() => duplicateWidget(context.instance.id)}
              onDelete={() => setDeleteTarget(context.instance)}
              onSetPinned={(pinned) => void runAction(async () => {
                await api.setManagedWidgetPinned(context.instance.id, pinned);
                await refreshDetail(context.instance.id);
              })}
              onUpdateWidget={updateWidget}
              onAddPane={() => void runAction(async () => {
                await api.createManagedWidgetPane(context.instance.id, null);
                await refreshDetail(context.instance.id);
              })}
              onUpdatePane={updatePane}
              onDeletePane={(pane) => void runAction(async () => {
                await api.deleteManagedWidgetPane(context.instance.id, pane.id);
                await refreshDetail(context.instance.id);
              })}
              onMovePane={movePane}
              onResetPaneSizes={() => void runAction(async () => {
                await api.resetManagedWidgetPaneWeights(context.instance.id);
                await refreshDetail(context.instance.id);
              })}
              onManageThemes={onManageThemes}
            />
          ) : (
            <div className="widget-manager-detail-empty">{t("widgetManager.selectWidget")}</div>
          )}
        </main>
        </div>

      {deleteTarget ? (
        <ConfirmDialog
          danger
          busy={busy}
          title={t("widgetManager.deleteTitle", { name: deleteTarget.name })}
          body={t("widgetManager.deleteBody")}
          confirmLabel={t("widgetManager.confirmDelete")}
          onCancel={() => setDeleteTarget(null)}
          onConfirm={deleteWidget}
        />
      ) : null}
      </section>
    </ManagementShell>
  );
}

type CardProps = {
  instance: WidgetInstance;
  current: boolean;
  selected: boolean;
  themes: WidgetTheme[];
  busy: boolean;
  onManage: () => void;
  onShow: () => void;
  onClose: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
};

function WidgetManagerCard({
  instance,
  current,
  selected,
  themes,
  busy,
  onManage,
  onShow,
  onClose,
  onDuplicate,
  onDelete,
}: CardProps) {
  const { t } = useI18n();
  const theme = themes.find((item) => item.id === (instance.defaultThemeId || "default"));
  return (
    <article className="widget-manager-card" data-selected={selected ? "true" : undefined}>
      <button className="widget-manager-card__summary" type="button" onClick={onManage}>
        <strong>{instance.name}</strong>
        <span>{instance.isPrimary ? t("widget.primary") : t("widgetManager.secondary")}{current ? ` · ${t("widget.current")}` : ""}</span>
        <span className="widget-manager-card__runtime">
          <i data-open={instance.isOpen ? "true" : undefined} />
          {instance.isOpen ? t("widgetManager.openState") : t("widgetManager.closedState")}
          {instance.pinned ? ` · ${t("widgetManager.pinnedState")}` : ` · ${t("widgetManager.normalState")}`}
        </span>
        <span>{layoutLabel(instance.layoutMode, t)} · {t("widgetManager.paneCount", { count: instance.paneCount })}</span>
        <span>{theme ? displayThemeName(theme, t) : t("theme.unavailable")}</span>
      </button>
      <div className="widget-manager-card__actions">
        <button className="text-button" type="button" disabled={busy} onClick={onShow}>
          <ExternalLink size={14} />
          {instance.isOpen ? t("widget.show") : t("widget.open")}
        </button>
        <button className="secondary-button" type="button" disabled={busy} onClick={onManage}>
          <Settings2 size={14} />
          {t("widgetManager.manage")}
        </button>
        {!instance.isPrimary && instance.isOpen ? (
          <button className="secondary-button" type="button" disabled={busy} onClick={onClose}>
            {t("widgetManager.closeWindow")}
          </button>
        ) : null}
        <button className="secondary-button" type="button" disabled={busy} onClick={onDuplicate}>
          <Copy size={14} />
          {t("widgetManager.duplicate")}
        </button>
        {!instance.isPrimary ? (
          <button
            className="icon-button icon-button--danger"
            type="button"
            disabled={busy}
            title={t("widgetManager.deleteWidget")}
            aria-label={t("widgetManager.deleteWidget")}
            onClick={onDelete}
          >
            <Trash2 size={14} />
          </button>
        ) : null}
      </div>
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

function layoutLabel(layout: WidgetInstance["layoutMode"], t: ReturnType<typeof useI18n>["t"]) {
  if (layout === "columns") return t("pane.layoutColumns");
  if (layout === "rows") return t("pane.layoutRows");
  if (layout === "grid") return t("pane.layoutGrid");
  return t("pane.layoutSingle");
}

function errorMessage(error: unknown, t: ReturnType<typeof useI18n>["t"]) {
  const message = error instanceof Error ? error.message : String(error);
  return localizeBackendMessage(message, t);
}
