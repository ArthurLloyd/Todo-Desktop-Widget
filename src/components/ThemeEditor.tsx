import { Copy, RotateCcw, Save, Trash2 } from "lucide-react";
import { useI18n, type TranslationKey } from "../lib/i18n";
import { InlineError } from "./InlineError";
import {
  type ThemeColorKey,
  type ThemeDraftError,
  type ThemeDraftErrors,
  type ThemeEditorDraft,
} from "../lib/themeEditor";
import { ColorField } from "./ColorField";
import { SliderField } from "./SliderField";
import { ThemePreview } from "./ThemePreview";

type Props = {
  draft: ThemeEditorDraft;
  initial: ThemeEditorDraft;
  errors: ThemeDraftErrors;
  isBuiltin: boolean;
  isNew: boolean;
  dirty: boolean;
  saving: boolean;
  saveError?: string | null;
  windowOpacity: number;
  onChange: (draft: ThemeEditorDraft) => void;
  onSave: () => void;
  onCancel: () => void;
  onDuplicate: () => void;
  onDelete: () => void;
};

const colorGroups: Array<{
  titleKey: TranslationKey;
  fields: Array<{ key: ThemeColorKey; labelKey: TranslationKey }>;
}> = [
  {
    titleKey: "themeEditor.baseColors",
    fields: [
      { key: "background", labelKey: "themeEditor.color.background" },
      { key: "foreground", labelKey: "themeEditor.color.foreground" },
      { key: "secondaryText", labelKey: "themeEditor.color.secondaryText" },
      { key: "accent", labelKey: "themeEditor.color.accent" },
    ],
  },
  {
    titleKey: "themeEditor.taskColors",
    fields: [
      { key: "taskOverdue", labelKey: "themeEditor.color.taskOverdue" },
      { key: "taskDueToday", labelKey: "themeEditor.color.taskDueToday" },
      { key: "taskImportant", labelKey: "themeEditor.color.taskImportant" },
      { key: "taskPending", labelKey: "themeEditor.color.taskPending" },
      { key: "taskCompleted", labelKey: "themeEditor.color.taskCompleted" },
    ],
  },
];

export function ThemeEditor({
  draft,
  initial,
  errors,
  isBuiltin,
  isNew,
  dirty,
  saving,
  saveError,
  windowOpacity,
  onChange,
  onSave,
  onCancel,
  onDuplicate,
  onDelete,
}: Props) {
  const { t } = useI18n();
  const readOnly = isBuiltin && !isNew;
  const canSave = !readOnly && dirty && Object.keys(errors).length === 0 && !saving;

  function updateColor(key: ThemeColorKey, value: string) {
    onChange({ ...draft, config: { ...draft.config, [key]: value } });
  }

  return (
    <div className="theme-editor">
      <div className="theme-editor__form">
        <label className="theme-name-field">
          <span>{t("themeEditor.name")}</span>
          <input
            type="text"
            value={draft.name}
            disabled={readOnly}
            aria-invalid={Boolean(errors.name)}
            onChange={(event) => onChange({ ...draft, name: event.target.value })}
          />
          {errors.name ? (
            <small className="theme-field-error">{themeErrorMessage(errors.name, t)}</small>
          ) : null}
        </label>

        {readOnly ? <div className="theme-readonly-note">{t("themeEditor.builtinReadonly")}</div> : null}

        {colorGroups.map((group) => (
          <fieldset className="theme-editor__group" key={group.titleKey} disabled={readOnly}>
            <legend>{t(group.titleKey)}</legend>
            {group.fields.map(({ key, labelKey }) => (
              <ColorField
                key={key}
                label={t(labelKey)}
                value={draft.config[key]}
                error={errors[key] ? themeErrorMessage(errors[key], t) : undefined}
                disabled={readOnly}
                onChange={(value) => updateColor(key, value)}
              />
            ))}
          </fieldset>
        ))}

        <fieldset className="theme-editor__group" disabled={readOnly}>
          <legend>{t("themeEditor.effects")}</legend>
          <SliderField
            label={t("themeEditor.opacity")}
            value={Math.round(draft.config.backgroundOpacity * 100)}
            min={0}
            max={100}
            step={1}
            displayValue={`${Math.round(draft.config.backgroundOpacity * 100)}%`}
            error={errors.backgroundOpacity ? themeErrorMessage(errors.backgroundOpacity, t) : undefined}
            disabled={readOnly}
            onChange={(value) => onChange({
              ...draft,
              config: { ...draft.config, backgroundOpacity: value / 100 },
            })}
          />
          <SliderField
            label={t("themeEditor.blur")}
            value={draft.config.blurPx}
            min={0}
            max={64}
            step={1}
            displayValue={`${draft.config.blurPx} px`}
            error={errors.blurPx ? themeErrorMessage(errors.blurPx, t) : undefined}
            disabled={readOnly}
            onChange={(value) => onChange({
              ...draft,
              config: { ...draft.config, blurPx: value },
            })}
          />
          <SliderField
            label={t("themeEditor.radius")}
            value={draft.config.borderRadiusPx}
            min={0}
            max={64}
            step={1}
            displayValue={`${draft.config.borderRadiusPx} px`}
            error={errors.borderRadiusPx ? themeErrorMessage(errors.borderRadiusPx, t) : undefined}
            disabled={readOnly}
            onChange={(value) => onChange({
              ...draft,
              config: { ...draft.config, borderRadiusPx: value },
            })}
          />
        </fieldset>

        <div className="theme-editor__actions">
          <button className="secondary-button" type="button" onClick={onDuplicate}>
            <Copy size={15} />
            {t("themeEditor.duplicate")}
          </button>
          {!readOnly ? (
            <>
              <button className="secondary-button" type="button" disabled={!dirty || saving} onClick={onCancel}>
                <RotateCcw size={15} />
                {isNew ? t("common.cancel") : t("themeEditor.reset")}
              </button>
              <button className="text-button" type="button" disabled={!canSave} onClick={onSave}>
                <Save size={15} />
                {saving ? t("themeEditor.saving") : t("themeEditor.save")}
              </button>
              {!isNew ? (
                <button className="details-delete-button" type="button" disabled={saving} onClick={onDelete}>
                  <Trash2 size={15} />
                  {t("themeEditor.delete")}
                </button>
              ) : null}
            </>
          ) : null}
        </div>
        {saveError ? <InlineError message={saveError} /> : null}
      </div>

      <ThemePreview
        config={draft.config}
        fallbackConfig={initial.config}
        windowOpacity={windowOpacity}
      />
    </div>
  );
}

function themeErrorMessage(
  error: ThemeDraftError,
  t: ReturnType<typeof useI18n>["t"],
) {
  if (error === "required") return t("themeEditor.errorRequired");
  if (error === "tooLong") return t("themeEditor.errorNameLength");
  if (error === "invalidColor") return t("themeEditor.errorColor");
  return t("themeEditor.errorRange");
}
