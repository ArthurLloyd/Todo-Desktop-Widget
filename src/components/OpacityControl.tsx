import { useEffect, useRef, useState } from "react";
import { useI18n } from "../lib/i18n";
import { InlineError } from "./InlineError";
import { parseOpacityPercent } from "../lib/opacity";

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
  const [editing, setEditing] = useState(false);
  const [percent, setPercent] = useState("");
  const [invalid, setInvalid] = useState(false);
  const editActive = useRef(false);
  const percentInput = useRef<HTMLInputElement>(null);
  const valueButton = useRef<HTMLButtonElement>(null);
  const restoreFocus = useRef(false);
  const running = useRef(false);
  const pending = useRef<{ value: number; save: typeof onChange } | null>(null);
  useEffect(() => { if (!running.current) setDraft(value); }, [value]);
  useEffect(() => {
    if (editing) { percentInput.current?.focus(); percentInput.current?.select(); }
    else if (restoreFocus.current) { restoreFocus.current = false; valueButton.current?.focus(); }
  }, [editing]);

  function startEditing() {
    setPercent(String(Math.round(draft * 100)));
    setInvalid(false);
    editActive.current = true;
    setEditing(true);
  }

  function commitEditing(returnFocus = false) {
    if (!editActive.current) return;
    const next = parseOpacityPercent(percent);
    if (next === null) { setInvalid(true); return; }
    editActive.current = false;
    restoreFocus.current = returnFocus;
    setEditing(false);
    setInvalid(false);
    void update(next);
  }

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
    <div className="opacity-control__field">
      <div className="opacity-control__heading">
        <label htmlFor="window-opacity">{t("settings.opacityLabel")}</label>
        {editing ? <span className="opacity-control__editor">
          <input type="text" inputMode="numeric" value={percent} aria-label={t("settings.opacityInput")}
            aria-invalid={invalid} aria-describedby={invalid ? "opacity-input-error" : undefined}
            ref={percentInput}
            onChange={(event) => { setPercent(event.target.value); setInvalid(false); }}
            onBlur={() => commitEditing()}
            onKeyDown={(event) => {
              if (event.key === "Enter") { event.preventDefault(); commitEditing(true); }
              if (event.key === "Escape") { event.preventDefault(); editActive.current = false; restoreFocus.current = true; setEditing(false); setInvalid(false); }
            }} /><span aria-hidden="true">%</span>
        </span> : <button ref={valueButton} type="button" className="opacity-control__value" title={t("settings.opacityEdit")}
          aria-label={t("settings.opacityEditValue", { value: Math.round(draft * 100) })}
          onDoubleClick={startEditing} onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") { event.preventDefault(); startEditing(); }
          }}>{Math.round(draft * 100)}%</button>}
      </div>
      <input id="window-opacity" aria-label={t("settings.opacityLabel")} type="range" min={0} max={100} step={1} value={Math.round(draft * 100)}
        onChange={(event) => void update(Number(event.target.value) / 100)} />
    </div>
    <div className="opacity-control__scale">
      <span>{t("settings.transparent")}</span>
      <span className="settings-save-status" role="status">{saving ? t("settings.saving") : error ? t("settings.saveFailed") : t("settings.autoSave")}</span>
      <span>{t("settings.opaque")}</span>
    </div>
    {invalid ? <p id="opacity-input-error" className="opacity-control__error" role="alert">{t("settings.opacityInvalid")}</p> : null}
    {error ? <InlineError message={error} /> : null}
  </div>;
}
