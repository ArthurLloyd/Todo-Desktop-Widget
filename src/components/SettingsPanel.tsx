import { ExternalLink, LogIn, LogOut, Plus, RotateCcw, Trash2, UserCircle, X } from "lucide-react";
import { localizeBackendMessage, useI18n } from "../lib/i18n";
import type { AuthStatus, Settings, TaskList, WidgetContext, WidgetInstance, WidgetLayoutMode } from "../lib/types";

type Props = {
  open: boolean;
  auth: AuthStatus;
  settings: Settings;
  loginNotice?: string | null;
  onClose: () => void;
  onLogin: () => void;
  onLogout: () => void;
  onChange: (settings: Partial<Settings>) => void;
  onEnsureLists: () => void;
  widgetInstances: WidgetInstance[];
  currentWidgetId: string | null;
  onCreateWidget: () => void;
  onOpenWidget: (instanceId: string) => void;
  onCloseWidget?: () => void;
  currentWidget: WidgetContext | null;
  taskLists: TaskList[];
  configError?: string | null;
  onRenameWidget: (name: string) => void;
  onChangeLayout: (layoutMode: WidgetLayoutMode) => void;
  onAddPane: () => void;
  onSelectPaneList: (paneId: string, listId: string) => void;
  onDeletePane: (paneId: string) => void;
};

export function SettingsPanel({
  open,
  auth,
  settings,
  loginNotice,
  onClose,
  onLogin,
  onLogout,
  onChange,
  onEnsureLists,
  widgetInstances,
  currentWidgetId,
  onCreateWidget,
  onOpenWidget,
  onCloseWidget,
  currentWidget,
  taskLists,
  configError,
  onRenameWidget,
  onChangeLayout,
  onAddPane,
  onSelectPaneList,
  onDeletePane,
}: Props) {
  const { t } = useI18n();

  if (!open) return null;

  return (
    <aside className="settings-panel">
      <div className="settings-panel__header">
        <h2>{t("settings.title")}</h2>
        <button className="icon-button" title={t("settings.close")} onClick={onClose}>
          <X size={16} />
        </button>
      </div>

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
          min={70}
          max={100}
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
          onChange={(event) => onChange({ autostart: event.target.checked })}
        />
      </label>

      <label className="settings-row">
        <span>{t("settings.showCompleted")}</span>
        <input
          type="checkbox"
          checked={settings.showCompleted}
          onChange={(event) => onChange({ showCompleted: event.target.checked })}
        />
      </label>

      <label className="settings-row">
        <span>{t("settings.debugConsole")}</span>
        <input
          type="checkbox"
          checked={settings.debugMode}
          onChange={(event) => onChange({ debugMode: event.target.checked })}
        />
      </label>

      <section className="settings-group widget-manager">
        <div className="widget-manager__heading">
          <strong>{t("widget.desktopWidgets")}</strong>
          <button className="text-button" onClick={onCreateWidget}>
            <Plus size={15} />
            {t("widget.newWindow")}
          </button>
        </div>
        <div className="widget-manager__list">
          {widgetInstances.map((instance) => (
            <div className="widget-manager__item" key={instance.id}>
              <span>{instance.name}{instance.isPrimary ? ` · ${t("widget.primary")}` : ""}</span>
              {!instance.isPrimary && (
                <button
                  className="text-button"
                  disabled={instance.id === currentWidgetId}
                  onClick={() => onOpenWidget(instance.id)}
                >
                  <ExternalLink size={14} />
                  {instance.id === currentWidgetId ? t("widget.current") : instance.isOpen ? t("widget.show") : t("widget.open")}
                </button>
              )}
            </div>
          ))}
        </div>
        {onCloseWidget && (
          <button className="secondary-button" onClick={onCloseWidget}>
            {t("widget.closeWindow")}
          </button>
        )}
      </section>

      {currentWidget && (
        <section className="settings-group pane-manager">
          <strong>{t("pane.currentWidget")}</strong>
          <label className="settings-row">
            <span>{t("pane.windowName")}</span>
            <input
              key={currentWidget.instance.name}
              className="settings-text-input"
              defaultValue={currentWidget.instance.name}
              aria-label={t("pane.windowName")}
              onBlur={(event) => onRenameWidget(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") event.currentTarget.blur();
              }}
            />
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
          {currentWidget.instance.layoutMode === "single" && currentWidget.panes.length > 1 && (
            <span className="settings-hint">{t("pane.singleHint")}</span>
          )}
          <div className="pane-manager__heading">
            <span>{t("pane.sections")}</span>
            <button className="text-button" type="button" onClick={onAddPane}>
              <Plus size={15} />
              {t("pane.add")}
            </button>
          </div>
          <div className="pane-manager__list">
            {currentWidget.panes.map((pane) => {
              const list = taskLists.find((item) => item.id === pane.listId);
              const name = pane.titleOverride || list?.displayName ||
                (pane.listId ? t("pane.listUnavailable") : t("pane.selectList"));
              return (
                <div className="pane-manager__item" key={pane.id}>
                  <span>{name}</span>
                  <select
                    aria-label={t("pane.listSelector", { name })}
                    value={list?.id || ""}
                    onChange={(event) => {
                      if (event.target.value) onSelectPaneList(pane.id, event.target.value);
                    }}
                  >
                    <option value="" disabled>
                      {pane.listId ? t("pane.listUnavailable") : t("pane.selectList")}
                    </option>
                    {taskLists.map((item) => (
                      <option key={item.id} value={item.id}>{item.displayName}</option>
                    ))}
                  </select>
                  <button
                    className="icon-button icon-button--danger"
                    type="button"
                    disabled={currentWidget.panes.length <= 1}
                    title={currentWidget.panes.length <= 1 ? t("pane.atLeastOne") : t("pane.remove")}
                    aria-label={t("pane.removeNamed", { name })}
                    onClick={() => onDeletePane(pane.id)}
                  >
                    <Trash2 size={14} />
                  </button>
                </div>
              );
            })}
          </div>
          {configError && (
            <span className="settings-error">{localizeBackendMessage(configError, t)}</span>
          )}
        </section>
      )}

      <button className="secondary-button" onClick={onEnsureLists}>
        <RotateCcw size={16} />
        {t("settings.syncRemoteLists")}
      </button>
    </aside>
  );
}
