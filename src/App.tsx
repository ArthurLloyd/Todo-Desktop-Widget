import { useCallback, useEffect, useMemo, useReducer, useRef, useState, type CSSProperties } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { api } from "./lib/api";
import type {
  AuthStatus,
  AppInfo,
  ListRole,
  Settings,
  SortMode,
  SyncStatus,
  Task,
  TaskList as TodoList,
  TaskPatch,
  WidgetConfigChanged,
  WidgetContext,
  WidgetInstance,
  WidgetLayoutMode,
  WidgetPane,
  WidgetPaneWeightInput,
  WidgetTheme,
  WidgetThemeInput,
} from "./lib/types";
import { WidgetShell } from "./components/WidgetShell";
import { TitleBar } from "./components/TitleBar";
import { FooterStatus } from "./components/FooterStatus";
import { SettingsPanel } from "./components/SettingsPanel";
import { ThemeManager } from "./components/ThemeManager";
import { WidgetManager } from "./components/WidgetManager";
import { QuickAddOverlay } from "./components/QuickAddOverlay";
import { TaskDetailsPanel } from "./components/TaskDetailsPanel";
import { WidgetContextMenu } from "./components/WidgetContextMenu";
import { PaneLayout } from "./components/PaneLayout";
import { WidgetPaneView } from "./components/WidgetPaneView";
import { compareTaskDueDates } from "./lib/dates";
import { I18nProvider, useI18n } from "./lib/i18n";
import {
  createThemeMap,
  resolvePaneThemeOverride,
  resolveWidgetTheme,
  themeToCssVariables,
} from "./lib/themeRuntime";
import { deriveManagementTheme } from "./lib/managementTheme";
import { reduceWidgetView } from "./lib/widgetView";
import { classifyStartupPresentation } from "./lib/widgetPresentation";
import { FirstRunState, WidgetLoadError, WidgetLoadingState } from "./components/WidgetStartupState";

const defaultStatus: SyncStatus = { state: "idle" };
const defaultSettings: Settings = {
  alwaysOnTop: false,
  opacity: 0.78,
  syncIntervalMinutes: 3,
  autostart: false,
  debugMode: false,
  showCompleted: false,
  language: "system",
  fontFamily: "system",
  fontScale: 1,
};

export default function App() {
  const [windowLabel, setWindowLabel] = useState<string | null>(null);
  const route = getRoute();
  const view = route.get("view");
  const taskId = route.get("taskId");

  useEffect(() => {
    try {
      setWindowLabel(getCurrentWindow().label);
    } catch {
      setWindowLabel(view === "details" ? "task-details-preview" : "main");
    }
  }, []);

  if (view === "details" || windowLabel?.startsWith("task-details-")) {
    return <TaskDetailsWindow taskId={taskId} windowLabel={windowLabel} />;
  }

  if (windowLabel === null) {
    return null;
  }

  return <WidgetApp />;
}

function WidgetApp() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [taskLists, setTaskLists] = useState<TodoList[]>([]);
  const [inboxCount, setInboxCount] = useState(0);
  const [syncStatus, setSyncStatus] = useState<SyncStatus>(defaultStatus);
  const [auth, setAuth] = useState<AuthStatus>({ loggedIn: false });
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [appInfo, setAppInfo] = useState<AppInfo | null>(null);
  const [initialLoading, setInitialLoading] = useState(true);
  const [initialLoadError, setInitialLoadError] = useState<string | null>(null);
  const [widgetContext, setWidgetContext] = useState<WidgetContext | null>(null);
  const [widgetThemes, setWidgetThemes] = useState<WidgetTheme[]>([]);
  const widgetInstanceIdRef = useRef<string | null>(null);
  const [widgetInstances, setWidgetInstances] = useState<WidgetInstance[]>([]);
  const [widgetConfigError, setWidgetConfigError] = useState<string | null>(null);
  const [view, dispatchView] = useReducer(reduceWidgetView, "widget");
  const [quickAddOpen, setQuickAddOpen] = useState(false);
  const [loginNotice, setLoginNotice] = useState<string | null>(null);
  const [sortMode, setSortMode] = useState<SortMode>(() => readSortMode());
  const [contextMenu, setContextMenu] = useState<{ open: boolean; x: number; y: number }>({
    open: false,
    x: 0,
    y: 0,
  });
  const [completingIds, setCompletingIds] = useState<Set<string>>(() => new Set());

  const sortedTasks = useMemo(() => sortTasks(tasks, sortMode), [sortMode, tasks]);
  const themeMap = useMemo(() => createThemeMap(widgetThemes), [widgetThemes]);
  const widgetTheme = useMemo(
    () => resolveWidgetTheme(themeMap, widgetContext?.instance.defaultThemeId),
    [themeMap, widgetContext?.instance.defaultThemeId],
  );
  const widgetThemeStyle = useMemo(
    () => widgetTheme ? themeToCssVariables(widgetTheme.config, settings.opacity) : undefined,
    [settings.opacity, widgetTheme],
  );
  const managementThemeStyle = useMemo(
    () => deriveManagementTheme(widgetTheme?.config),
    [widgetTheme],
  );
  const startupPresentation = classifyStartupPresentation({
    loading: initialLoading,
    loadError: initialLoadError,
    loggedIn: auth.loggedIn,
    taskCount: tasks.length,
  });

  const refreshTaskData = useCallback(async () => {
    const [snapshot, nextStatus, nextAuth, nextSettings] = await Promise.all([
      api.getLocalWidgetSnapshot(),
      api.getSyncStatus(),
      api.getAuthStatus(),
      api.getSettings(),
    ]);
    setTaskLists(snapshot.taskLists);
    setTasks(snapshot.tasks);
    setSyncStatus(nextStatus);
    setInboxCount(snapshot.inboxCount);
    setAuth(nextAuth);
    setSettings(nextSettings);
  }, []);

  const reloadWidgetContext = useCallback(async () => {
    const [nextContext, nextThemes] = await Promise.all([
      api.getCurrentWidgetContext(),
      api.listWidgetThemes(),
    ]);
    widgetInstanceIdRef.current = nextContext.instance.id;
    setWidgetContext(nextContext);
    setWidgetThemes(nextThemes);
  }, []);

  const reloadWidgetInstances = useCallback(async () => {
    const nextInstances = await api.listWidgetInstances();
    setWidgetInstances(nextInstances);
    return nextInstances;
  }, []);

  const loadInitialState = useCallback(async () => {
    setInitialLoading(true);
    setInitialLoadError(null);
    try {
      const [, , , nextAppInfo] = await Promise.all([
        refreshTaskData(),
        reloadWidgetContext(),
        reloadWidgetInstances(),
        api.getAppInfo(),
      ]);
      setAppInfo(nextAppInfo);
    } catch (error) {
      setInitialLoadError(error instanceof Error ? error.message : String(error));
    } finally {
      setInitialLoading(false);
    }
  }, [refreshTaskData, reloadWidgetContext, reloadWidgetInstances]);

  useEffect(() => {
    void loadInitialState();
  }, [loadInitialState]);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let unlistenConfig: (() => void) | undefined;
    let unlistenThemes: (() => void) | undefined;
    let unlistenOpenSettings: (() => void) | undefined;
    void listen("local-tasks-changed", () => void refreshTaskData()).then((nextUnlisten) => {
      unlisten = nextUnlisten;
    });
    void listen<WidgetConfigChanged>("widget-config-changed", (event) => {
      void reloadWidgetInstances();
      if (event.payload.widgetInstanceId === widgetInstanceIdRef.current) {
        void reloadWidgetContext();
      }
    }).then((nextUnlisten) => {
      unlistenConfig = nextUnlisten;
    });
    void listen("theme-definitions-changed", () => {
      void reloadWidgetContext();
    }).then((nextUnlisten) => {
      unlistenThemes = nextUnlisten;
    });
    void listen("open-settings", () => {
      setContextMenu((current) => ({ ...current, open: false }));
      dispatchView({ type: "open-settings" });
    }).then((nextUnlisten) => {
      unlistenOpenSettings = nextUnlisten;
    });
    return () => {
      unlisten?.();
      unlistenConfig?.();
      unlistenThemes?.();
      unlistenOpenSettings?.();
    };
  }, [refreshTaskData, reloadWidgetContext, reloadWidgetInstances]);

  useEffect(() => {
    if (view !== "widget") return;
    function onKeyDown(event: KeyboardEvent) {
      const quickAddShortcut = (event.ctrlKey && event.altKey) || (event.metaKey && event.altKey);
      if (quickAddShortcut && event.key.toLowerCase() === "t") {
        event.preventDefault();
        setQuickAddOpen(true);
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [view]);

  useEffect(() => {
    if (view !== "widget") return;
    function onContextMenu(event: MouseEvent) {
      event.preventDefault();
      const menuWidth = 166;
      const menuHeight = 176;
      setContextMenu({
        open: true,
        x: Math.min(event.clientX, window.innerWidth - menuWidth - 8),
        y: Math.min(event.clientY, window.innerHeight - menuHeight - 8),
      });
    }

    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "Escape") {
        setContextMenu((current) => ({ ...current, open: false }));
      }
    }

    window.addEventListener("contextmenu", onContextMenu);
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("contextmenu", onContextMenu);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [view]);

  useEffect(() => {
    if (!loginNotice || auth.loggedIn) return;

    const interval = window.setInterval(() => void refreshTaskData(), 1500);
    return () => window.clearInterval(interval);
  }, [auth.loggedIn, loginNotice, refreshTaskData]);

  useEffect(() => {
    if (auth.loggedIn && loginNotice) {
      setLoginNotice(null);
      void refreshTaskData();
    }
  }, [auth.loggedIn, loginNotice, refreshTaskData]);

  async function handlePaneAdd(pane: WidgetPane, title: string) {
    if (!pane.listId) return;
    const task = await api.addTaskToList(title, pane.listId);
    setTasks((current) => [task, ...current]);
    await refreshTaskData();
  }

  async function handleQuickAdd(title: string, listRole: ListRole) {
    const task = await api.addTask(title, listRole);
    setTasks((current) => [task, ...current]);
    await refreshTaskData();
  }

  async function handleComplete(taskId: string) {
    setCompletingIds((current) => new Set(current).add(taskId));
    setTasks((current) =>
      current.map((task) => (task.id === taskId ? { ...task, status: "completed", dirty: true } : task)),
    );

    try {
      await api.completeTask(taskId);
      await refreshTaskData();
    } finally {
      setCompletingIds((current) => {
        const next = new Set(current);
        next.delete(taskId);
        return next;
      });
    }
  }

  async function handleSync() {
    setSyncStatus((current) => ({ ...current, state: "syncing", message: "Syncing" }));
    try {
      const nextStatus = await api.syncNow();
      setSyncStatus(nextStatus);
      await refreshTaskData();
    } catch (error) {
      setSyncStatus({
        state: "error",
        message: error instanceof Error ? error.message : String(error),
      });
    }
  }

  async function handleLogin() {
    setLoginNotice("Opening Microsoft sign-in");
    try {
      const login = await api.login();
      setLoginNotice(login.message);
      setSyncStatus({
        state: "auth_required",
        message: "Waiting for Microsoft login",
      });
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLoginNotice(message);
      setSyncStatus({
        state: "auth_required",
        message,
      });
    }
  }

  async function handleSettingsChange(next: Partial<Settings>) {
    const updated = await api.updateSettings(next);
    setSettings(updated);
  }

  async function handleTogglePinned() {
    if (!widgetContext) return;
    const pinned = !widgetContext.instance.pinned;
    await api.setWindowMode(pinned ? "pinned" : "normal");
    setWidgetContext((current) => current ? {
      ...current,
      instance: { ...current.instance, pinned },
    } : current);
    setWidgetInstances((current) => current.map((instance) =>
      instance.id === widgetContext.instance.id ? { ...instance, pinned } : instance,
    ));
  }

  async function handleSelectPaneList(paneId: string, listId: string) {
    setWidgetConfigError(null);
    try {
      await api.setCurrentWidgetPaneList(paneId, listId);
      await reloadWidgetContext();
    } catch (error) {
      setWidgetConfigError(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleLayoutChange(layoutMode: WidgetLayoutMode) {
    setWidgetConfigError(null);
    try {
      await api.updateCurrentWidgetLayout(layoutMode);
      await reloadWidgetContext();
    } catch (error) {
      setWidgetConfigError(error instanceof Error ? error.message : String(error));
    }
  }

  async function handlePaneWeightsChange(weights: WidgetPaneWeightInput[]) {
    const panes = await api.setCurrentWidgetPaneWeights(weights);
    setWidgetContext((current) => current ? { ...current, panes } : current);
  }

  async function handleWidgetThemeChange(themeId: string) {
    setWidgetConfigError(null);
    try {
      await api.setCurrentWidgetTheme(themeId);
      await reloadWidgetContext();
    } catch (error) {
      setWidgetConfigError(error instanceof Error ? error.message : String(error));
    }
  }

  async function handleCreateTheme(input: WidgetThemeInput) {
    const created = await api.createWidgetTheme(input);
    await reloadWidgetContext();
    return created;
  }

  async function handleUpdateTheme(id: string, input: WidgetThemeInput) {
    const updated = await api.updateWidgetTheme(id, input);
    await reloadWidgetContext();
    return updated;
  }

  async function handleDeleteTheme(id: string) {
    await api.deleteWidgetTheme(id);
    await reloadWidgetContext();
  }

  async function hideWindow() {
    await api.setWindowMode("hidden");
  }

  function openSettings() {
    setContextMenu((current) => ({ ...current, open: false }));
    dispatchView({ type: "open-settings" });
  }

  return (
    <I18nProvider language={settings.language}>
      <WidgetShell
        opacity={settings.opacity}
        fontFamily={settings.fontFamily}
        fontScale={settings.fontScale}
        themeStyle={widgetThemeStyle}
      >
        {view === "widget" ? (
          <>
        <TitleBar
          isPinned={widgetContext?.instance.pinned || false}
          widgetName={widgetContext?.instance.name || "Todo"}
          sortMode={sortMode}
          syncStatus={syncStatus}
          onDragStart={() => void api.startDrag()}
          onSelectSort={(nextSortMode) => {
            setSortMode(nextSortMode);
            window.localStorage.setItem("todo-sort-mode", nextSortMode);
          }}
          onRefresh={handleSync}
          onTogglePinned={() => void handleTogglePinned()}
          onOpenSettings={openSettings}
        />

        {startupPresentation === "loading" ? <WidgetLoadingState /> : null}
        {startupPresentation === "load-error" && initialLoadError ? (
          <WidgetLoadError message={initialLoadError} onRetry={() => void loadInitialState()} />
        ) : null}
        {startupPresentation === "first-run" ? (
          <FirstRunState onLogin={() => void handleLogin()} onOpenSettings={openSettings} />
        ) : null}
        {startupPresentation === "ready" && widgetContext ? (
          <PaneLayout
            layoutMode={widgetContext.instance.layoutMode}
            panes={widgetContext.panes}
            onPersistWeights={handlePaneWeightsChange}
            renderPane={(pane) => {
              const list = taskLists.find((item) => item.id === pane.listId) || null;
              const showCompleted = pane.showCompleted ?? settings.showCompleted;
              const paneTasks = sortedTasks
                .filter((task) => task.listId === pane.listId)
                .filter((task) => showCompleted || task.status !== "completed")
                .slice(0, pane.maxTasks ?? undefined);
              const paneTheme = resolvePaneThemeOverride(themeMap, pane.themeId);
              const paneThemeStyle = paneTheme
                ? themeToCssVariables(paneTheme.config, settings.opacity)
                : undefined;
              return (
                <WidgetPaneView
                  key={pane.id}
                  pane={pane}
                  list={list}
                  lists={taskLists}
                  tasks={paneTasks}
                  completingIds={completingIds}
                  themeStyle={paneThemeStyle}
                  onSelectList={(paneId, listId) => void handleSelectPaneList(paneId, listId)}
                  onAdd={handlePaneAdd}
                  onComplete={handleComplete}
                  onOpenDetails={(task) => void api.openTaskDetails(task.id)}
                />
              );
            }}
          />
        ) : null}
        {startupPresentation === "ready" ? (
          <FooterStatus inboxCount={inboxCount} syncStatus={syncStatus} onRetry={() => void handleSync()} />
        ) : null}

        <WidgetContextMenu
          open={contextMenu.open}
          x={contextMenu.x}
          y={contextMenu.y}
          isPinned={widgetContext?.instance.pinned || false}
          onClose={() => setContextMenu((current) => ({ ...current, open: false }))}
          onRefresh={() => void handleSync()}
          onTogglePinned={() => void handleTogglePinned()}
          onOpenSettings={openSettings}
          onHide={() => void hideWindow()}
        />

        <QuickAddOverlay open={quickAddOpen} onAdd={handleQuickAdd} onClose={() => setQuickAddOpen(false)} />
          </>
        ) : null}

      {view === "settings" ? <SettingsPanel
        managementThemeStyle={managementThemeStyle}
        auth={auth}
        settings={settings}
        appInfo={appInfo}
        loginNotice={loginNotice}
        onClose={() => dispatchView({ type: "close" })}
        onLogin={() => void handleLogin()}
        onLogout={() => void api.logout().then(() => refreshTaskData())}
        onChange={handleSettingsChange}
        onEnsureLists={() => void handleSync()}
        currentWidget={widgetContext}
        widgetThemes={widgetThemes}
        configError={widgetConfigError}
        onTogglePinned={() => void handleTogglePinned()}
        onChangeLayout={(layoutMode) => void handleLayoutChange(layoutMode)}
        onChangeWidgetTheme={(themeId) => void handleWidgetThemeChange(themeId)}
        onOpenWidgetManager={() => dispatchView({ type: "open-widget-manager" })}
        onOpenThemeManager={() => dispatchView({ type: "open-theme-manager" })}
      /> : null}

      {view === "widget-manager" ? (
        <WidgetManager
          managementThemeStyle={managementThemeStyle}
          instances={widgetInstances}
          currentWidgetId={widgetContext?.instance.id || null}
          taskLists={taskLists}
          themes={widgetThemes}
          onBack={() => dispatchView({ type: "back" })}
          onClose={() => dispatchView({ type: "close" })}
          onRefreshInstances={reloadWidgetInstances}
          onManageThemes={() => dispatchView({ type: "open-theme-manager" })}
        />
      ) : null}

      {view === "theme-manager" ? (
        <ThemeManager
          managementThemeStyle={managementThemeStyle}
          themes={widgetThemes}
          windowOpacity={settings.opacity}
          onBack={() => dispatchView({ type: "back" })}
          onClose={() => dispatchView({ type: "close" })}
          onCreate={handleCreateTheme}
          onUpdate={handleUpdateTheme}
          onDelete={handleDeleteTheme}
        />
      ) : null}
      </WidgetShell>
    </I18nProvider>
  );
}

function getRoute() {
  const params = new URLSearchParams(window.location.search);
  const hash = window.location.hash.startsWith("#") ? window.location.hash.slice(1) : "";
  if (hash.startsWith("details")) {
    const [, query = ""] = hash.split("?");
    const hashParams = new URLSearchParams(query);
    hashParams.set("view", "details");
    return hashParams;
  }
  return params;
}

function readSortMode(): SortMode {
  const value = window.localStorage.getItem("todo-sort-mode");
  if (
    value === "due" ||
    value === "created" ||
    value === "modified" ||
    value === "importance" ||
    value === "title" ||
    value === "reminder"
  ) {
    return value;
  }
  return "default";
}

function sortTasks(tasks: Task[], sortMode: SortMode) {
  const next = [...tasks];
  next.sort((a, b) => {
    const statusOrder = completedRank(a) - completedRank(b);
    if (statusOrder !== 0) return statusOrder;

    if (a.status === "completed" && b.status === "completed") {
      return compareOptionalDateDesc(a.completedDateTime || a.modifiedAt, b.completedDateTime || b.modifiedAt);
    }

    if (sortMode === "title") return a.title.localeCompare(b.title);
    if (sortMode === "importance") return importanceRank(b) - importanceRank(a) || fallbackOrder(a, b);
    if (sortMode === "created") return compareOptionalDateDesc(a.createdAt, b.createdAt) || compareTaskDueDates(a, b);
    if (sortMode === "modified") return compareOptionalDateDesc(a.modifiedAt, b.modifiedAt) || compareTaskDueDates(a, b);
    if (sortMode === "reminder") return compareOptionalDate(a.reminderDateTime, b.reminderDateTime) || compareTaskDueDates(a, b);

    if (sortMode === "due" || sortMode === "default") {
      return compareTaskDueDates(a, b) || fallbackOrder(a, b);
    }

    return 0;
  });

  return next;
}

function completedRank(task: Task) {
  return task.status === "completed" ? 1 : 0;
}

function importanceRank(task: Task) {
  if (task.importance === "high") return 3;
  if (task.importance === "normal" || !task.importance) return 2;
  return 1;
}

function compareOptionalDate(left?: string | null, right?: string | null) {
  const leftTime = parseOptionalDate(left);
  const rightTime = parseOptionalDate(right);
  return leftTime - rightTime;
}

function compareOptionalDateDesc(left?: string | null, right?: string | null) {
  const leftTime = parseOptionalDate(left);
  const rightTime = parseOptionalDate(right);
  if (leftTime === Number.POSITIVE_INFINITY && rightTime === Number.POSITIVE_INFINITY) return 0;
  if (leftTime === Number.POSITIVE_INFINITY) return 1;
  if (rightTime === Number.POSITIVE_INFINITY) return -1;
  return rightTime - leftTime;
}

function parseOptionalDate(value?: string | null) {
  if (!value) return Number.POSITIVE_INFINITY;
  const time = new Date(value).getTime();
  return Number.isFinite(time) ? time : Number.POSITIVE_INFINITY;
}

function fallbackOrder(a: Task, b: Task) {
  return compareOptionalDateDesc(a.modifiedAt || a.createdAt, b.modifiedAt || b.createdAt);
}

function TaskDetailsWindow({ taskId, windowLabel }: { taskId: string | null; windowLabel: string | null }) {
  const [resolvedTaskId, setResolvedTaskId] = useState<string | null>(taskId);
  const [task, setTask] = useState<Task | null>(null);
  const [taskLists, setTaskLists] = useState<TodoList[]>([]);
  const [settings, setSettings] = useState<Settings>(defaultSettings);
  const [saving, setSaving] = useState(false);
  const [deleting, setDeleting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const taskList = taskLists.find((list) => list.id === task?.listId);

  useEffect(() => {
    function onContextMenu(event: MouseEvent) {
      event.preventDefault();
    }

    window.addEventListener("contextmenu", onContextMenu);
    return () => window.removeEventListener("contextmenu", onContextMenu);
  }, []);

  useEffect(() => {
    if (taskId) {
      setResolvedTaskId(taskId);
      return;
    }

    if (!windowLabel) return;

    void api
      .getTaskIdForWindow(windowLabel)
      .then((nextTaskId) => {
        if (nextTaskId) {
          setResolvedTaskId(nextTaskId);
          setError(null);
        } else {
          setError("details.resolveError");
        }
      })
      .catch((resolveError) => {
        setError(resolveError instanceof Error ? resolveError.message : String(resolveError));
      });
  }, [taskId, windowLabel]);

  const load = useCallback(async () => {
    if (!resolvedTaskId) return;
    try {
      const [nextTask, nextLists, nextSettings] = await Promise.all([
        api.getTask(resolvedTaskId),
        api.getTaskLists(),
        api.getSettings(),
      ]);
      setTask(nextTask);
      setTaskLists(nextLists);
      setSettings(nextSettings);
      setError(null);
    } catch (loadError) {
      setError(loadError instanceof Error ? loadError.message : String(loadError));
    }
  }, [resolvedTaskId]);

  useEffect(() => {
    void load();
  }, [load]);

  async function handleSave(nextTaskId: string, patch: TaskPatch) {
    setSaving(true);
    try {
      const updated = await api.updateTask(nextTaskId, patch);
      setTask(updated);
      await load();
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : String(saveError));
    } finally {
      setSaving(false);
    }
  }

  async function handleDelete(nextTaskId: string) {
    setDeleting(true);
    setError(null);
    try {
      await api.deleteTask(nextTaskId);
      closeWindow();
    } catch (deleteError) {
      console.error("Failed to delete task locally", deleteError);
      setError("details.deleteFailed");
      throw deleteError;
    } finally {
      setDeleting(false);
    }
  }

  function closeWindow() {
    const label = windowLabel || getCurrentWindow().label;
    void api.closeTaskDetailsWindow(label).catch(() => getCurrentWindow().close());
  }

  return (
    <I18nProvider language={settings.language}>
      <TaskDetailsWindowFrame
        settings={settings}
        task={task}
        taskList={taskList}
        saving={saving}
        deleting={deleting}
        error={error}
        onClose={closeWindow}
        onSave={handleSave}
        onDelete={handleDelete}
      />
    </I18nProvider>
  );
}

function TaskDetailsWindowFrame({
  settings,
  task,
  taskList,
  saving,
  deleting,
  error,
  onClose,
  onSave,
  onDelete,
}: {
  settings: Settings;
  task: Task | null;
  taskList?: TodoList;
  saving: boolean;
  deleting: boolean;
  error: string | null;
  onClose: () => void;
  onSave: (taskId: string, patch: TaskPatch) => Promise<void>;
  onDelete: (taskId: string) => Promise<void>;
}) {
  const { t } = useI18n();
  const displayError =
    error === "details.resolveError" || error === "details.deleteFailed" ? t(error) : error;

  return (
    <main
      className="details-window"
      data-font={settings.fontFamily}
      style={{ "--font-scale": settings.fontScale } as CSSProperties}
    >
      {task ? (
        <TaskDetailsPanel
          task={task}
          taskList={taskList}
          saving={saving}
          deleting={deleting}
          windowed
          onClose={onClose}
          onSave={onSave}
          onDelete={onDelete}
        />
      ) : (
        <div className="details-loading">
          <span>{displayError || t("details.loading")}</span>
        </div>
      )}
      {displayError && task ? <div className="details-error">{displayError}</div> : null}
    </main>
  );
}
