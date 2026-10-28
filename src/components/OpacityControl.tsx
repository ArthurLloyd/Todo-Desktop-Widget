import { useEffect, useRef, useState } from "react";
import { useI18n } from "../lib/i18n";
import { InlineError } from "./InlineError";

// Serialize saves and coalesce intermediate slider values. A slow save must
// never overwrite the final value or a different selected window.
export function OpacityControl({ value, onChange }: {
  value: number;
  onChange: (value: number) => Promise<void>;
}) {
  const { t } = useI18n();
  const [draft, setDraft] = useState(value);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const running = useRef(false);
  const pending = useRef<{ value: number; save: typeof onChange } | null>(null);
  useEffect(() => { if (!running.current) setDraft(value); }, [value]);

  async function update(next: number) {
    setDraft(next);
    pending.current = { value: next, save: onChange };
    if (running.current) return;
    running.current = true;
    setSaving(true);
    setError(null);
    while (pending.current) {
      const job = pending.current;
      pending.current = null;
      try { await job.save(job.value); setError(null); }
      catch (cause) { setError(String(cause)); }
    }
    running.current = false;
    setSaving(false);
  }

  return <div className="opacity-control">
    <label className="opacity-control__field">
      <span>{t("settings.opacityLabel")}<output>{Math.round(draft * 100)}%</output></span>
      <input aria-label={t("settings.opacityLabel")} type="range" min={0} max={100} step={1} value={Math.round(draft * 100)}
        onChange={(event) => void update(Number(event.target.value) / 100)} />
    </label>
    <div className="opacity-control__scale">
      <span>{t("settings.transparent")}</span>
      <span className="settings-save-status" role="status">{saving ? t("settings.saving") : error ? t("settings.saveFailed") : t("settings.autoSave")}</span>
      <span>{t("settings.opaque")}</span>
    </div>
    {error ? <InlineError message={error} /> : null}
  </div>;
}
