import { Plus, X } from "lucide-react";
import { useRef, useState } from "react";
import { useI18n } from "../lib/i18n";
import { InlineError } from "./InlineError";

export function CreateListForm({ onCreate, enabled }: {
  onCreate: (name: string) => Promise<void>;
  enabled: boolean;
}) {
  const { t } = useI18n();
  const [open, setOpen] = useState(false);
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const trigger = useRef<HTMLButtonElement>(null);
  const [error, setError] = useState<string | null>(null);
  function close() { setOpen(false); setError(null); trigger.current?.focus(); }
  return <div className="create-list">
    <button ref={trigger} className="text-button" type="button" disabled={!enabled || busy}
      title={!enabled ? t("lists.loginRequired") : t("lists.create")} aria-label={t("lists.create")} aria-expanded={open}
      onClick={() => { setOpen(!open); setError(null); }}><Plus size={15} /><span>{t("lists.create")}</span></button>
    {open ? <form className="create-list__form" onSubmit={async (event) => {
      event.preventDefault();
      if (!name.trim() || busyRef.current) return;
      busyRef.current = true; setBusy(true); setError(null);
      try { await onCreate(name.trim()); setName(""); close(); }
      catch { setError(t("lists.createFailed")); }
      finally { busyRef.current = false; setBusy(false); }
    }}>
      <label><span>{t("lists.name")}</span><input autoFocus value={name} maxLength={255}
        disabled={busy} placeholder={t("lists.placeholder")} onChange={(event) => setName(event.target.value)}
        onKeyDown={(event) => { if (event.key === "Escape" && !busy) { event.stopPropagation(); close(); } }} /></label>
      <p>{t("lists.hint")}</p>
      <div className="create-list__actions"><button className="primary-button" disabled={busy || !name.trim()}>{busy ? t("lists.creating") : t("lists.createAndUse")}</button>
        <button className="icon-button" type="button" disabled={busy} aria-label={t("common.close")} onClick={close}><X size={16} /></button></div>
      {error ? <InlineError message={error} /> : null}
    </form> : null}
  </div>;
}
