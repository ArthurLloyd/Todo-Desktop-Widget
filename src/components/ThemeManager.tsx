import { Copy, Plus, X } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { useI18n } from "../lib/i18n";
import {
  cloneThemeDraft,
  draftFromTheme,
  nextAvailableThemeName,
  normalizeThemeDraft,
  themeDraftEquals,
  validateThemeDraft,
  type ThemeEditorDraft,
} from "../lib/themeEditor";
import type { WidgetTheme, WidgetThemeInput } from "../lib/types";
import { ThemeEditor } from "./ThemeEditor";

type Props = {
  themes: WidgetTheme[];
  windowOpacity: number;
  onClose: () => void;
  onCreate: (input: WidgetThemeInput) => Promise<WidgetTheme>;
  onUpdate: (id: string, input: WidgetThemeInput) => Promise<WidgetTheme>;
  onDelete: (id: string) => Promise<void>;
};

type PendingAction =
  | { kind: "close" }
  | { kind: "select"; themeId: string }
  | { kind: "new" }
  | { kind: "duplicate" };

export function ThemeManager({ themes, windowOpacity, onClose, onCreate, onUpdate, onDelete }: Props) {
  const { t } = useI18n();
  const firstTheme = themes.find((theme) => theme.id === "default") ?? themes[0];
  const [selectedId, setSelectedId] = useState<string | null>(firstTheme?.id ?? null);
  const [draft, setDraft] = useState<ThemeEditorDraft>(() => initialDraft(firstTheme));
  const [initial, setInitial] = useState<ThemeEditorDraft>(() => initialDraft(firstTheme));
  const [isNew, setIsNew] = useState(false);
  const [returnThemeId, setReturnThemeId] = useState<string | null>(firstTheme?.id ?? null);
  const [pendingAction, setPendingAction] = useState<PendingAction | null>(null);
  const [deletePending, setDeletePending] = useState(false);
  const [saving, setSaving] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const selectedTheme = themes.find((theme) => theme.id === selectedId) ?? null;
  const errors = useMemo(() => validateThemeDraft(draft), [draft]);
  const dirty = isNew || !themeDraftEquals(draft, initial);
  const builtins = useMemo(() => themes.filter((theme) => theme.isBuiltin), [themes]);
  const customs = useMemo(
    () => themes.filter((theme) => !theme.isBuiltin).sort((left, right) => left.name.localeCompare(right.name)),
    [themes],
  );

  useEffect(() => {
    if (isNew || dirty) return;
    const refreshed = themes.find((theme) => theme.id === selectedId) ??
      themes.find((theme) => theme.id === "default") ??
      themes[0];
    if (!refreshed) return;
    const nextDraft = draftFromTheme(refreshed);
    setSelectedId(refreshed.id);
    setReturnThemeId(refreshed.id);
    setDraft(nextDraft);
    setInitial(cloneThemeDraft(nextDraft));
  }, [dirty, isNew, selectedId, themes]);

  function requestAction(action: PendingAction) {
    if (dirty) setPendingAction(action);
    else performAction(action);
  }

  function performAction(action: PendingAction) {
    setPendingAction(null);
    setSaveError(null);
    if (action.kind === "close") {
      onClose();
      return;
    }
    if (action.kind === "select") {
      selectExisting(action.themeId);
      return;
    }
    if (action.kind === "new") {
      const source = themes.find((theme) => theme.id === "default") ?? themes[0];
      if (!source) return;
      beginCreate({
        name: nextAvailableThemeName(
          t("themeEditor.newThemeName"),
          themes.map((theme) => theme.name),
        ),
        config: { ...source.config },
      });
      return;
    }
    const source = themes.find((theme) => theme.id === selectedId);
    if (!source) return;
    beginCreate({
      name: nextAvailableThemeName(
        `${source.name} ${t("themeEditor.copySuffix")}`,
        themes.map((theme) => theme.name),
      ),
      config: { ...source.config },
    });
  }

  function selectExisting(themeId: string) {
    const theme = themes.find((item) => item.id === themeId);
    if (!theme) return;
    const nextDraft = draftFromTheme(theme);
    setSelectedId(theme.id);
    setReturnThemeId(theme.id);
    setDraft(nextDraft);
    setInitial(cloneThemeDraft(nextDraft));
    setIsNew(false);
  }

  function beginCreate(nextDraft: ThemeEditorDraft) {
    setReturnThemeId(selectedId ?? returnThemeId);
    setSelectedId(null);
    setDraft(nextDraft);
    setInitial(cloneThemeDraft(nextDraft));
    setIsNew(true);
  }

  async function save() {
    const normalized = normalizeThemeDraft(draft);
    setDraft(normalized);
    if (Object.keys(validateThemeDraft(normalized)).length > 0) return;
    setSaving(true);
    setSaveError(null);
    try {
      const input: WidgetThemeInput = { name: normalized.name, config: normalized.config };
      const saved = isNew
        ? await onCreate(input)
        : selectedTheme
          ? await onUpdate(selectedTheme.id, input)
          : null;
      if (!saved) return;
      const savedDraft = draftFromTheme(saved);
      setSelectedId(saved.id);
      setReturnThemeId(saved.id);
      setDraft(savedDraft);
      setInitial(cloneThemeDraft(savedDraft));
      setIsNew(false);
    } catch (error) {
      setSaveError(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  function cancelDraft() {
    setSaveError(null);
    if (isNew) {
      const fallbackId = returnThemeId ?? firstTheme?.id;
      if (fallbackId) selectExisting(fallbackId);
      return;
    }
    setDraft(cloneThemeDraft(initial));
  }

  async function confirmDelete() {
    if (!selectedTheme || selectedTheme.isBuiltin) return;
    setSaving(true);
    setSaveError(null);
    try {
      await onDelete(selectedTheme.id);
      setDeletePending(false);
      const fallback = themes.find((theme) => theme.id === "default") ??
        themes.find((theme) => theme.id !== selectedTheme.id);
      if (fallback) selectExisting(fallback.id);
    } catch (error) {
      setSaveError(error instanceof Error ? error.message : String(error));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="theme-manager" role="dialog" aria-modal="true" aria-label={t("themeEditor.managerTitle")}>
      <header className="theme-manager__header">
        <div>
          <strong>{t("themeEditor.managerTitle")}</strong>
          <span>{t("themeEditor.managerHint")}</span>
        </div>
        <button
          className="icon-button"
          type="button"
          title={t("common.close")}
          aria-label={t("common.close")}
          onClick={() => requestAction({ kind: "close" })}
        >
          <X size={16} />
        </button>
      </header>

      <div className="theme-manager__body">
        <aside className="theme-library">
          <button className="text-button" type="button" onClick={() => requestAction({ kind: "new" })}>
            <Plus size={15} />
            {t("themeEditor.newTheme")}
          </button>
          <ThemeGroup
            title={t("themeEditor.builtinThemes")}
            themes={builtins}
            selectedId={selectedId}
            onSelect={(themeId) => requestAction({ kind: "select", themeId })}
          />
          <ThemeGroup
            title={t("themeEditor.customThemes")}
            themes={customs}
            selectedId={selectedId}
            emptyLabel={t("themeEditor.noCustomThemes")}
            onSelect={(themeId) => requestAction({ kind: "select", themeId })}
          />
          <button className="secondary-button" type="button" disabled={!selectedTheme} onClick={() => requestAction({ kind: "duplicate" })}>
            <Copy size={15} />
            {t("themeEditor.duplicate")}
          </button>
        </aside>

        <ThemeEditor
          draft={draft}
          initial={initial}
          errors={errors}
          isBuiltin={Boolean(selectedTheme?.isBuiltin)}
          isNew={isNew}
          dirty={dirty}
          saving={saving}
          saveError={saveError}
          windowOpacity={windowOpacity}
          onChange={setDraft}
          onSave={() => void save()}
          onCancel={cancelDraft}
          onDuplicate={() => requestAction({ kind: "duplicate" })}
          onDelete={() => setDeletePending(true)}
        />
      </div>

      {pendingAction ? (
        <Confirmation
          title={t("themeEditor.discardTitle")}
          body={t("themeEditor.discardBody")}
          confirmLabel={t("themeEditor.discard")}
          onCancel={() => setPendingAction(null)}
          onConfirm={() => performAction(pendingAction)}
        />
      ) : null}
      {deletePending && selectedTheme ? (
        <Confirmation
          danger
          title={t("themeEditor.deleteTitle", { name: selectedTheme.name })}
          body={t("themeEditor.deleteBody")}
          confirmLabel={t("common.delete")}
          onCancel={() => setDeletePending(false)}
          onConfirm={() => void confirmDelete()}
        />
      ) : null}
    </div>
  );
}

function ThemeGroup({
  title,
  themes,
  selectedId,
  emptyLabel,
  onSelect,
}: {
  title: string;
  themes: WidgetTheme[];
  selectedId: string | null;
  emptyLabel?: string;
  onSelect: (themeId: string) => void;
}) {
  const { t } = useI18n();
  return (
    <section className="theme-library__group">
      <strong>{title}</strong>
      {themes.length === 0 ? <span className="settings-hint">{emptyLabel}</span> : null}
      {themes.map((theme) => (
        <button
          className="theme-library__item"
          data-selected={theme.id === selectedId ? "true" : undefined}
          type="button"
          key={theme.id}
          onClick={() => onSelect(theme.id)}
        >
          <span
            className="theme-library__swatch"
            style={{ background: theme.config.background, color: theme.config.accent }}
          >
            ●
          </span>
          <span>{displayThemeName(theme, t)}</span>
          {theme.isBuiltin ? <small>{t("themeEditor.builtinBadge")}</small> : null}
        </button>
      ))}
    </section>
  );
}

function displayThemeName(theme: WidgetTheme, t: ReturnType<typeof useI18n>["t"]) {
  if (theme.id === "default") return t("theme.default");
  if (theme.id === "ocean") return t("theme.ocean");
  if (theme.id === "violet") return t("theme.violet");
  if (theme.id === "forest") return t("theme.forest");
  return theme.name;
}

function Confirmation({
  title,
  body,
  confirmLabel,
  danger = false,
  onCancel,
  onConfirm,
}: {
  title: string;
  body: string;
  confirmLabel: string;
  danger?: boolean;
  onCancel: () => void;
  onConfirm: () => void;
}) {
  const { t } = useI18n();
  return (
    <div className="theme-confirmation" role="alertdialog" aria-modal="true" aria-label={title}>
      <div className="theme-confirmation__card">
        <strong>{title}</strong>
        <p>{body}</p>
        <div>
          <button className="secondary-button" type="button" onClick={onCancel}>{t("common.cancel")}</button>
          <button className={danger ? "danger-button" : "text-button"} type="button" onClick={onConfirm}>
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}

function initialDraft(theme?: WidgetTheme): ThemeEditorDraft {
  if (theme) return draftFromTheme(theme);
  return {
    name: "",
    config: {
      version: 1,
      background: "#F5F7FA",
      foreground: "#1F2937",
      secondaryText: "#6B7280",
      accent: "#2563EB",
      taskOverdue: "#DC2626",
      taskDueToday: "#2563EB",
      taskImportant: "#D97706",
      taskPending: "#6B7280",
      taskCompleted: "#64748B",
      backgroundOpacity: 0.78,
      blurPx: 24,
      borderRadiusPx: 18,
    },
  };
}
