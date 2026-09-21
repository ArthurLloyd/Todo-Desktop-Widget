import { LogIn, RefreshCw } from "lucide-react";
import { useI18n } from "../lib/i18n";
import { InlineError } from "./InlineError";

export function WidgetLoadingState() {
  const { t } = useI18n();
  return (
    <div className="widget-loading" aria-live="polite" aria-label={t("startup.loading")}>
      <span className="widget-loading__line widget-loading__line--title" />
      <span className="widget-loading__line" />
      <span className="widget-loading__line" />
      <span className="widget-loading__line widget-loading__line--short" />
    </div>
  );
}

export function WidgetLoadError({ message, onRetry }: { message: string; onRetry: () => void }) {
  const { t } = useI18n();
  return (
    <div className="widget-state">
      <InlineError message={t("startup.loadFailed")} onRetry={onRetry} retryLabel={t("common.retry")} />
      <span className="widget-state__detail" title={message}>{t("startup.localDataSafe")}</span>
    </div>
  );
}

export function FirstRunState({ onLogin, onOpenSettings }: { onLogin: () => void; onOpenSettings: () => void }) {
  const { t } = useI18n();
  return (
    <section className="first-run-state">
      <div className="first-run-state__icon"><LogIn size={20} /></div>
      <h2>{t("startup.welcome")}</h2>
      <p>{t("startup.welcomeBody")}</p>
      <ol>
        <li>{t("startup.stepConnect")}</li>
        <li>{t("startup.stepChooseList")}</li>
        <li>{t("startup.stepPin")}</li>
      </ol>
      <div className="first-run-state__actions">
        <button className="text-button" type="button" onClick={onLogin}>
          <LogIn size={15} /> {t("settings.login")}
        </button>
        <button className="secondary-button" type="button" onClick={onOpenSettings}>
          <RefreshCw size={14} /> {t("title.settings")}
        </button>
      </div>
    </section>
  );
}
