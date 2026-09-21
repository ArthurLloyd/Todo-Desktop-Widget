import { LayoutDashboard, LogIn, LogOut, Palette, RotateCcw, UserCircle } from "lucide-react";
import { useState } from "react";
import { localizeBackendMessage, useI18n } from "../lib/i18n";
import type { ManagementThemeCssVariables } from "../lib/managementTheme";
import type {
  AuthStatus,
  AppInfo,
  Settings,
  WidgetContext,
  WidgetLayoutMode,
  WidgetTheme,
} from "../lib/types";
import { ManagementShell } from "./ManagementShell";
import { InlineError } from "./InlineError";

type Props = {
  managementThemeStyle: ManagementThemeCssVariables;
  auth: AuthStatus;
  settings: Settings;
  appInfo: AppInfo | null;
  loginNotice?: string | null;
  onClose: () => void;
  onLogin: () => void;
  onLogout: () => void;
  onChange: (settings: Partial<Settings>) => Promise<void>;
  onEnsureLists: () => void;
  currentWidget: WidgetContext | null;
  widgetThemes: WidgetTheme[];
  configError?: string | null;
  onTogglePinned: () => void;
  onChangeLayout: (layoutMode: WidgetLayoutMode) => void;
  onChangeWidgetTheme: (themeId: string) => void;
  onOpenWidgetManager: () => void;
  onOpenThemeManager: () => void;
};

export function SettingsPanel({
  managementThemeStyle,
  auth,
  settings,
  appInfo,
  loginNotice,
  onClose,
  onLogin,
  onLogout,
  onChange,
  onEnsureLists,
  currentWidget,
  widgetThemes,
  configError,
  onTogglePinned,
  onChangeLayout,
  onChangeWidgetTheme,
  onOpenWidgetManager,
  onOpenThemeManager,
}: Props) {
  const { t } = useI18n();
  const [autostartBusy, setAutostartBusy] = useState(false);
  const [autostartError, setAutostartError] = useState<string | null>(null);

  async function updateAutostart(enabled: boolean) {
    if (autostartBusy) return;
    setAutostartBusy(true);
    setAutostartError(null);
    try {
      await onChange({ autostart: enabled });
    } catch (error) {
      setAutostartError(error instanceof Error ? error.message : String(error));
    } finally {
      setAutostartBusy(false);
    }
  }

  return (
    <ManagementShell title={t("settings.title")} themeStyle={managementThemeStyle} onClose={onClose}>
      <div className="settings-panel">

      <section
        className={
          auth.loggedIn
            ? "settings-group settings-account settings-account--signed-in"
            : "settings-group settings-account settings-account--signed-out"
        }
      >
        <UserCircle size={30} />
        <div>
          <strong>{auth.loggedIn ? auth.account || t("settings.msConnected") : t("settings.notSignedIn")}</strong>
          <span>
            {(loginNotice && localizeBackendMessage(loginNotice, t)) ||
              (auth.loggedIn ? t("settings.connectedHint") : t("settings.signedOutHint"))}
          </span>
        </div>
        <button className="text-button" onClick={auth.loggedIn ? onLogout : onLogin}>
          {auth.loggedIn ? <LogOut size={16} /> : <LogIn size={16} />}
          {auth.loggedIn ? t("settings.logout") : t("settings.login")}
        </button>
      </section>

      <label className="settings-row settings-row--stacked">
        <span>{t("settings.opacity", { value: Math.round(settings.opacity * 100) })}</span>
        <input
          min={0}
          max={100}
          step={1}
          type="range"
          value={Math.round(settings.opacity * 100)}
          onChange={(event) => onChange({ opacity: Number(event.target.value) / 100 })}
        />
      </label>

      <label className="settings-row">
        <span>{t("settings.language")}</span>
        <select
          value={settings.language}
          onChange={(event) => onChange({ language: event.target.value as Settings["language"] })}
        >
          <option value="system">{t("settings.languageSystem")}</option>
          <option value="en">{t("settings.languageEnglish")}</option>
          <option value="zh-CN">{t("settings.languageChinese")}</option>
        </select>
      </label>

      <label className="settings-row">
        <span>{t("settings.font")}</span>
        <select
          value={settings.fontFamily}
          onChange={(event) => onChange({ fontFamily: event.target.value as Settings["fontFamily"] })}
        >
          <option value="system">{t("settings.fontSystem")}</option>
          <option value="compact">{t("settings.fontCompact")}</option>
          <option value="serif">{t("settings.fontSerif")}</option>
          <option value="mono">{t("settings.fontMono")}</option>
        </select>
      </label>

      <label className="settings-row settings-row--stacked">
        <span>{t("settings.textSize", { value: Math.round(settings.fontScale * 100) })}</span>
        <input
          min={90}
          max={120}
          type="range"
          value={Math.round(settings.fontScale * 100)}
          onChange={(event) => onChange({ fontScale: Number(event.target.value) / 100 })}
        />
      </label>

      <label className="settings-row">
        <span>{t("settings.syncInterval")}</span>
        <select
          value={settings.syncIntervalMinutes}
          onChange={(event) => onChange({ syncIntervalMinutes: Number(event.target.value) })}
        >
          <option value={1}>{t("settings.minutes", { count: 1 })}</option>
          <option value={3}>{t("settings.minutes", { count: 3 })}</option>
          <option value={5}>{t("settings.minutes", { count: 5 })}</option>
          <option value={10}>{t("settings.minutes", { count: 10 })}</option>
        </select>
      </label>

      <label className="settings-row">
        <span>{t("settings.startWithOs")}</span>
        <input
          type="checkbox"
          checked={settings.autostart}
          disabled={autostartBusy}
          onChange={(event) => void updateAutostart(event.target.checked)}
        />
      </label>
      {autostartError ? <InlineError message={autostartError} /> : null}

      <label className="settings-row">
        <span>{t("settings.showCompleted")}</span>
        <input
          type="checkbox"
          checked={settings.showCompleted}
          onChange={(event) => onChange({ showCompleted: event.target.checked })}
        />
      </label>

      {import.meta.env.DEV ? (
        <label className="settings-row">
          <span>{t("settings.debugConsole")}</span>
          <input
            type="checkbox"
            checked={settings.debugMode}
            onChange={(event) => void onChange({ debugMode: event.target.checked })}
          />
        </label>
      ) : null}

      <section className="settings-group widget-manager-entry-group">
        <div>
          <strong>{t("widgetManager.managerTitle")}</strong>
          <span>{t("widgetManager.settingsHint")}</span>
        </div>
        <button className="secondary-button" type="button" onClick={onOpenWidgetManager}>
          <LayoutDashboard size={16} />
          {t("widgetManager.manageDesktopWidgets")}
        </button>
      </section>

      {currentWidget && (
        <section className="settings-group current-widget-settings">
          <strong>{t("pane.currentWidget")}</strong>
          <div className="current-widget-settings__name">
            <span>{currentWidget.instance.name}</span>
            <small>{currentWidget.instance.isPrimary ? t("widget.primary") : t("widgetManager.secondary")}</small>
          </div>
          <label className="settings-row">
            <span>{t("widgetManager.pinToDesktop")}</span>
            <input type="checkbox" checked={currentWidget.instance.pinned} onChange={onTogglePinned} />
          </label>
          <label className="settings-row">
            <span>{t("pane.layout")}</span>
            <select
              value={currentWidget.instance.layoutMode}
              onChange={(event) => onChangeLayout(event.target.value as WidgetLayoutMode)}
            >
              <option value="single">{t("pane.layoutSingle")}</option>
              <option value="columns">{t("pane.layoutColumns")}</option>
              <option value="rows">{t("pane.layoutRows")}</option>
              <option value="grid">{t("pane.layoutGrid")}</option>
            </select>
          </label>
          <button
            className="secondary-button theme-manager-entry"
            type="button"
            onClick={onOpenThemeManager}
          >
            <Palette size={15} />
            {t("themeEditor.manageThemes")}
          </button>
          <label className="settings-row">
            <span>{t("theme.windowTheme")}</span>
            <select
              value={
                widgetThemes.some(
                  (theme) => theme.id === (currentWidget.instance.defaultThemeId || "default"),
                )
                  ? currentWidget.instance.defaultThemeId || "default"
                  : "__missing__"
              }
              onChange={(event) => onChangeWidgetTheme(event.target.value)}
            >
              {!widgetThemes.some(
                (theme) => theme.id === (currentWidget.instance.defaultThemeId || "default"),
              ) && (
                <option value="__missing__" disabled>{t("theme.unavailable")}</option>
              )}
              {widgetThemes.map((theme) => (
                <option key={theme.id} value={theme.id}>{displayThemeName(theme, t)}</option>
              ))}
            </select>
          </label>
          {currentWidget.instance.layoutMode === "single" && currentWidget.panes.length > 1 && (
            <span className="settings-hint">{t("pane.singleHint")}</span>
          )}
          {configError && (
            <InlineError message={localizeBackendMessage(configError, t)} />
          )}
        </section>
      )}

      <button className="secondary-button" onClick={onEnsureLists}>
        <RotateCcw size={16} />
        {t("settings.syncRemoteLists")}
      </button>

      {appInfo ? (
        <section className="settings-group about-card">
          <div>
            <strong>{t("about.title")}</strong>
            <span>{appInfo.name}</span>
          </div>
          <dl>
            <div><dt>{t("about.version")}</dt><dd>{appInfo.version}</dd></div>
            <div><dt>{t("about.platform")}</dt><dd>{appInfo.platform} · {appInfo.architecture}</dd></div>
          </dl>
        </section>
      ) : null}
      </div>
    </ManagementShell>
  );
}

function displayThemeName(theme: WidgetTheme, t: ReturnType<typeof useI18n>["t"]) {
  if (theme.id === "default") return t("theme.default");
  if (theme.id === "ocean") return t("theme.ocean");
  if (theme.id === "violet") return t("theme.violet");
  if (theme.id === "forest") return t("theme.forest");
  return theme.name;
}
