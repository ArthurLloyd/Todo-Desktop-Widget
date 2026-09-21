export type ListRole = "today" | "inbox" | "this_week";
export type SortMode = "default" | "due" | "created" | "modified" | "importance" | "title" | "reminder";
export type LanguageSetting = "system" | "en" | "zh-CN";

export type Task = {
  id: string;
  graphId?: string | null;
  listId: string;
  title: string;
  status: "notStarted" | "completed";
  importance?: "low" | "normal" | "high" | null;
  note?: string | null;
  dueDateTime?: string | null;
  reminderDateTime?: string | null;
  completedDateTime?: string | null;
  timeZone?: string | null;
  isReminderOn?: boolean;
  recurrence?: Recurrence | null;
  createdAt?: string | null;
  modifiedAt?: string | null;
  dirty?: boolean;
};

export type RecurrenceMode = "none" | "daily" | "weekly" | "monthly" | "yearly";

export type Recurrence = {
  pattern?: {
    type?: string;
    interval?: number;
    daysOfWeek?: string[];
    firstDayOfWeek?: string;
    dayOfMonth?: number;
    month?: number;
  };
  range?: {
    type?: string;
    startDate?: string;
  };
};

export type TaskPatch = {
  title: string;
  importance?: "low" | "normal" | "high" | null;
  note?: string | null;
  dueDateTime?: string | null;
  reminderDateTime?: string | null;
  timeZone?: string | null;
  isReminderOn: boolean;
  recurrence?: Recurrence | null;
};

export type TaskList = {
  id: string;
  displayName: string;
  role?: ListRole | null;
};

export type SyncStatus = {
  state: "idle" | "syncing" | "error" | "offline" | "auth_required";
  lastSyncedAt?: string | null;
  message?: string | null;
};

export type Settings = {
  alwaysOnTop: boolean;
  opacity: number;
  syncIntervalMinutes: number;
  autostart: boolean;
  debugMode: boolean;
  showCompleted: boolean;
  language: LanguageSetting;
  fontFamily: "system" | "compact" | "serif" | "mono";
  fontScale: number;
};

export type WidgetLayoutMode = "single" | "columns" | "rows" | "grid";

export type WidgetThemeConfig = {
  version: 1;
  background: string;
  foreground: string;
  secondaryText: string;
  accent: string;
  taskOverdue: string;
  taskDueToday: string;
  taskImportant: string;
  taskPending: string;
  taskCompleted: string;
  backgroundOpacity: number;
  blurPx: number;
  borderRadiusPx: number;
};

export type WidgetInstance = {
  id: string;
  name: string;
  layoutMode: WidgetLayoutMode;
  defaultThemeId?: string | null;
  isPrimary: boolean;
  isOpen: boolean;
  pinned: boolean;
  paneCount: number;
  windowX?: number | null;
  windowY?: number | null;
  windowWidth?: number | null;
  windowHeight?: number | null;
  createdAt: string;
  updatedAt: string;
};

export type WidgetContext = {
  instance: WidgetInstance;
  panes: WidgetPane[];
};

export type WidgetTaskSnapshot = {
  taskLists: TaskList[];
  tasks: Task[];
  inboxCount: number;
};

export type WidgetConfigChanged = {
  widgetInstanceId: string;
};

export type WidgetInstanceInput = {
  name: string;
  layoutMode: WidgetLayoutMode;
  defaultThemeId?: string | null;
};

export type WidgetPane = {
  id: string;
  widgetInstanceId: string;
  listId?: string | null;
  titleOverride?: string | null;
  themeId?: string | null;
  sortOrder: number;
  showCompleted?: boolean | null;
  maxTasks?: number | null;
  sizeWeight: number;
  createdAt: string;
  updatedAt: string;
};

export type WidgetPaneInput = {
  widgetInstanceId: string;
  listId?: string | null;
  titleOverride?: string | null;
  themeId?: string | null;
  sortOrder: number;
  showCompleted?: boolean | null;
  maxTasks?: number | null;
  sizeWeight: number;
};

export type WidgetPaneWeightInput = {
  paneId: string;
  sizeWeight: number;
};

export type WidgetTheme = {
  id: string;
  name: string;
  config: WidgetThemeConfig;
  isBuiltin: boolean;
  createdAt: string;
  updatedAt: string;
};

export type WidgetThemeInput = {
  name: string;
  config: WidgetThemeConfig;
};

export type AuthStatus = {
  loggedIn: boolean;
  account?: string | null;
};

export type LoginStart = {
  authUrl: string;
  message: string;
};
