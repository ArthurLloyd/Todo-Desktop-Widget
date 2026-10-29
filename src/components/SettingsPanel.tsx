import { LayoutDashboard, LogIn, LogOut, Palette, RotateCcw, UserCircle } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { localizeBackendMessage, useI18n } from "../lib/i18n";
import type { ManagementThemeCssVariables } from "../lib/managementTheme";
import type {
  AuthStatus,
  AppInfo,
  Settings,
  WidgetContext,
  WidgetInstance,
  WidgetLayoutMode,
  WidgetTheme,
} from "../lib/types";
import { ManagementShell } from "./ManagementShell";
import { OpacityControl } from "./OpacityControl";
import { InlineError } from "./InlineError";
import { SettingsThemePicker } from "./SettingsThemePicker";

const UPSTREAM_PROJECT_URL = "https://github.com/Chenzo1001/Microsoft-Todo-Desktop-Widget";
const IMPROVEMENT_AUTHOR = "涂风";

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
  instances: WidgetInstance[];
  selectedWidgetId: string;
  onSelectWidget: (id: string) => void;
  windowOpacity: number;
  onOpacityChange: (opacity: number) => Promise<void>;
  widgetThemes: WidgetTheme[];
  configError?: string | null;
  onTogglePinned: () => void;
  onChangeLayout: (layoutMode: WidgetLayoutMode) => void;
  onChangeWidgetTheme: (themeId: string) => void;
  onOpenWidgetManager: () => void;
  onOpenThemeManager: () => void;
};

export function SettingsPanel({
  managementThemeStyle, auth, settings, appInfo, loginNotice, onClose, onLogin, onLogout,
  onChange, onEnsureLists, currentWidget, instances, selectedWidgetId, onSelectWidget,
  windowOpacity, onOpacityChange, widgetThemes, configError, onTogglePinned, onChangeLayout,
  onChangeWidgetTheme, onOpenWidgetManager, onOpenThemeManager,
}: Props) {
  const { t } = useI18n();
  const [tab, setTab] = useState<"window" | "personalization" | "general">("window");
  const [themeBusy, setThemeBusy] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);
  const [autostartBusy, setAutostartBusy] = useState(false);
  const panel = useRef<HTMLDivElement>(null);
  const configured = appInfo?.microsoftClientConfigured ?? true;
  useEffect(() => { panel.current?.closest(".management-shell__body")?.scrollTo(0, 0); }, [tab]);

  async function save(patch: Partial<Settings>) {
    setSaveError(null);
    if (patch.autostart !== undefined) setAutostartBusy(true);
    if (patch.settingsTheme !== undefined) setThemeBusy(true);
    try { await onChange(patch); }
    catch (error) { setSaveError(String(error)); }
    finally { setAutostartBusy(false); setThemeBusy(false); }
  }

  const tabs = <nav className="preferences-tabs" role="tablist" aria-label={t("settings.title")}
    onKeyDown={(event) => {
      if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
      event.preventDefault();
      const order = ["window", "personalization", "general"] as const;
      const next = event.key === "Home" ? order[0] : event.key === "End" ? order[2]
        : order[(order.indexOf(tab) + (event.key === "ArrowRight" ? 1 : 2)) % order.length];
      setTab(next);
      event.currentTarget.querySelector<HTMLButtonElement>(`#settings-tab-${next}`)?.focus();
    }}>
    <button type="button" role="tab" id="settings-tab-window" aria-controls="settings-content" aria-selected={tab === "window"} tabIndex={tab === "window" ? 0 : -1} onClick={() => setTab("window")}><LayoutDashboard size={17} />{t("settings.windowTab")}</button>
    <button type="button" role="tab" id="settings-tab-personalization" aria-controls="settings-content" aria-selected={tab === "personalization"} tabIndex={tab === "personalization" ? 0 : -1} onClick={() => setTab("personalization")}><Palette size={17} />{t("settings.personalizationTab")}</button>
    <button type="button" role="tab" id="settings-tab-general" aria-controls="settings-content" aria-selected={tab === "general"} tabIndex={tab === "general" ? 0 : -1} onClick={() => setTab("general")}><UserCircle size={17} />{t("settings.generalTab")}</button>
  </nav>;

  return <ManagementShell title={t("settings.title")} themeStyle={managementThemeStyle} onClose={onClose} toolbar={tabs} hideHeader>
    <div ref={panel} className="preferences-panel" id="settings-content" role="tabpanel" aria-labelledby={`settings-tab-${tab}`}>
      {configError ? <InlineError message={localizeBackendMessage(configError, t)} /> : null}
      {saveError ? <InlineError message={saveError} /> : null}
      {tab === "window" ? <>
        <SettingField id="editing-window" label={t("settings.editingWindow")} description={t("settings.windowHint")}>
          <select id="editing-window" value={selectedWidgetId} onChange={(event) => onSelectWidget(event.target.value)}>
            {instances.map((item) => <option key={item.id} value={item.id}>{item.name}{item.isOpen ? "" : ` · ${t("widgetManager.closedState")}`}</option>)}
          </select>
        </SettingField>
        {currentWidget ? <>
          <section className="preferences-group" aria-labelledby="appearance-heading">
            <h2 id="appearance-heading">{t("settings.appearanceHeading")}</h2>
            <OpacityControl key={currentWidget.instance.id} value={windowOpacity} onChange={onOpacityChange} />
            <p className="preferences-help">{t("settings.opacityHint")}</p>
            <SettingField id="window-theme" label={t("theme.windowTheme")}>
              <select id="window-theme" value={widgetThemes.some((theme) => theme.id === (currentWidget.instance.defaultThemeId || "default")) ? currentWidget.instance.defaultThemeId || "default" : "__missing__"} onChange={(event) => onChangeWidgetTheme(event.target.value)}>
                {!widgetThemes.some((theme) => theme.id === (currentWidget.instance.defaultThemeId || "default")) ? <option value="__missing__" disabled>{t("theme.unavailable")}</option> : null}
                {widgetThemes.map((theme) => <option key={theme.id} value={theme.id}>{displayThemeName(theme, t)}</option>)}
              </select>
            </SettingField>
            <div className="preferences-actions"><button className="secondary-button" onClick={onOpenThemeManager}><Palette size={16} />{t("themeEditor.manageThemes")}</button></div>
          </section>
          <section className="preferences-group" aria-labelledby="behavior-heading">
            <h2 id="behavior-heading">{t("settings.behaviorHeading")}</h2>
            <SettingField id="pin-window" label={t("widgetManager.pinToDesktop")} description={t("settings.pinHint")}>
              <input id="pin-window" role="switch" type="checkbox" checked={currentWidget.instance.pinned} onChange={onTogglePinned} />
            </SettingField>
            <SettingField id="window-layout" label={t("pane.layout")} description={t("settings.layoutHint")}>
              <select id="window-layout" value={currentWidget.instance.layoutMode} onChange={(event) => onChangeLayout(event.target.value as WidgetLayoutMode)}>
                <option value="single">{t("pane.layoutSingle")}</option><option value="columns">{t("pane.layoutColumns")}</option><option value="rows">{t("pane.layoutRows")}</option><option value="grid">{t("pane.layoutGrid")}</option>
              </select>
            </SettingField>
            {currentWidget.instance.layoutMode === "single" && currentWidget.panes.length > 1 ? <p className="preferences-help">{t("pane.singleHint")}</p> : null}
            <div className="preferences-actions"><button className="secondary-button" onClick={onOpenWidgetManager}><LayoutDashboard size={16} />{t("widgetManager.manageDesktopWidgets")}</button></div>
          </section>
        </> : null}
      </> : tab === "personalization" ? <SettingsThemePicker value={settings.settingsTheme} themes={widgetThemes} disabled={themeBusy}
        onChange={(settingsTheme) => void save({ settingsTheme })} /> : <>
        <section className="preferences-account" aria-label={t("settings.accountHeading")}>
          <UserCircle size={28} aria-hidden="true" />
          <div><strong>{auth.loggedIn ? auth.account || t("settings.msConnected") : t("settings.notSignedIn")}</strong><p>{loginNotice ? localizeBackendMessage(loginNotice, t) : auth.loggedIn ? t("settings.connectedHint") : t("settings.signedOutHint")}</p></div>
          <button className="secondary-button" disabled={!auth.loggedIn && !configured} onClick={auth.loggedIn ? onLogout : onLogin}>{auth.loggedIn ? <LogOut size={16} /> : <LogIn size={16} />}{auth.loggedIn ? t("settings.logout") : t("settings.login")}</button>
        </section>
        {!auth.loggedIn && !configured ? <InlineError message={t("settings.microsoftClientNotConfigured")} /> : null}
        <p className="preferences-help">{t("settings.generalHint")}</p>
        <section className="preferences-group" aria-labelledby="display-heading">
          <h2 id="display-heading">{t("settings.displayHeading")}</h2>
          <SettingField id="app-language" label={t("settings.language")}>
            <select id="app-language" value={settings.language} onChange={(event) => void save({ language: event.target.value as Settings["language"] })}><option value="system">{t("settings.languageSystem")}</option><option value="zh-CN">{t("settings.languageChinese")}</option><option value="en">{t("settings.languageEnglish")}</option></select>
          </SettingField>
          <SettingField id="app-font" label={t("settings.font")}>
            <select id="app-font" value={settings.fontFamily} onChange={(event) => void save({ fontFamily: event.target.value as Settings["fontFamily"] })}><option value="system">{t("settings.fontSystem")}</option><option value="compact">{t("settings.fontCompact")}</option><option value="serif">{t("settings.fontSerif")}</option><option value="mono">{t("settings.fontMono")}</option></select>
          </SettingField>
          <SettingField id="app-text-size" label={t("settings.textSize", { value: Math.round(settings.fontScale * 100) })}>
            <input id="app-text-size" min={90} max={120} type="range" value={Math.round(settings.fontScale * 100)} onChange={(event) => void save({ fontScale: Number(event.target.value) / 100 })} />
          </SettingField>
          <SettingField id="show-completed" label={t("settings.showCompleted")}>
            <input id="show-completed" role="switch" type="checkbox" checked={settings.showCompleted} onChange={(event) => void save({ showCompleted: event.target.checked })} />
          </SettingField>
        </section>
        <section className="preferences-group" aria-labelledby="sync-heading">
          <h2 id="sync-heading">{t("settings.syncHeading")}</h2>
          <SettingField id="sync-interval" label={t("settings.syncInterval")}>
            <select id="sync-interval" value={settings.syncIntervalMinutes} onChange={(event) => void save({ syncIntervalMinutes: Number(event.target.value) })}>{[1, 3, 5, 10].map((minutes) => <option key={minutes} value={minutes}>{t("settings.minutes", { count: minutes })}</option>)}</select>
          </SettingField>
          <SettingField id="app-autostart" label={t("settings.startWithOs")}>
            <input id="app-autostart" role="switch" type="checkbox" checked={settings.autostart} disabled={autostartBusy} onChange={(event) => void save({ autostart: event.target.checked })} />
          </SettingField>
          {import.meta.env.DEV ? <SettingField id="debug-console" label={t("settings.debugConsole")}><input id="debug-console" type="checkbox" checked={settings.debugMode} onChange={(event) => void save({ debugMode: event.target.checked })} /></SettingField> : null}
          <div className="preferences-actions"><button className="secondary-button" onClick={onEnsureLists}><RotateCcw size={16} />{t("settings.syncRemoteLists")}</button></div>
        </section>
        {appInfo ? <section className="preferences-group preferences-about" aria-labelledby="about-heading">
          <h2 id="about-heading">{t("about.title")}</h2>
          <p className="preferences-product-name">{appInfo.name}</p>
          <dl><dt>{t("about.version")}</dt><dd>{appInfo.version}</dd><dt>{t("about.platform")}</dt><dd>{appInfo.platform} · {appInfo.architecture}</dd><dt>{t("about.improvedBy")}</dt><dd>{IMPROVEMENT_AUTHOR}</dd></dl>
          <p className="preferences-help">{t("about.basedOn")} <a href={UPSTREAM_PROJECT_URL} target="_blank" rel="noreferrer noopener">Chenzo1001/Microsoft-Todo-Desktop-Widget</a></p>
        </section> : null}
      </>}
    </div>
  </ManagementShell>;
}

function SettingField({ id, label, description, children }: {
  id: string; label: string; description?: string; children: ReactNode;
}) {
  return <div className="preferences-field">
    <label htmlFor={id}><span>{label}</span>{description ? <small>{description}</small> : null}</label>
    <div className="preferences-field__control">{children}</div>
  </div>;
}

function displayThemeName(theme: WidgetTheme, t: ReturnType<typeof useI18n>["t"]) {
  if (theme.id === "default") return t("theme.default");
  if (theme.id === "ocean") return t("theme.ocean");
  if (theme.id === "violet") return t("theme.violet");
  if (theme.id === "forest") return t("theme.forest");
  return theme.name;
}
