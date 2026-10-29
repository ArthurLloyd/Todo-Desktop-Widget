import type { CSSProperties } from "react";
import { Check } from "lucide-react";
import { useI18n } from "../lib/i18n";
import { deriveSettingsTheme, resolveSettingsThemeId } from "../lib/settingsThemes";
import type { SettingsThemeId, WidgetTheme } from "../lib/types";
import gemArtwork from "../assets/themes/gem-stage.webp";
import fengArtwork from "../assets/themes/feng-noir.webp";
import chatgptArtwork from "../assets/themes/chatgpt-dragon.webp";
import deepseekArtwork from "../assets/themes/deepseek-girl.webp";

const featuredThemes = [
  { id: "gem", artwork: gemArtwork, signature: "G.E.M.", name: "settings.themeGem", description: "settings.themeGemDescription" },
  { id: "feng-bujue", artwork: fengArtwork, signature: "封不觉", name: "settings.themeFeng", description: "settings.themeFengDescription" },
  { id: "chatgpt", artwork: chatgptArtwork, signature: "ChatGPT", name: "settings.themeChatgpt", description: "settings.themeChatgptDescription" },
  { id: "deepseek", artwork: deepseekArtwork, signature: "DeepSeek", name: "settings.themeDeepseek", description: "settings.themeDeepseekDescription" },
] as const;

export function SettingsThemePicker({ value, themes, disabled, onChange }: {
  value: SettingsThemeId;
  themes: WidgetTheme[];
  disabled: boolean;
  onChange: (value: SettingsThemeId) => void;
}) {
  const { t } = useI18n();
  const selected = resolveSettingsThemeId(value);
  return <section className="settings-personalization" aria-labelledby="settings-theme-heading">
    <div className="settings-personalization__intro">
      <h2 id="settings-theme-heading">{t("settings.themeHeading")}</h2>
      <p>{t("settings.themeHint")}</p>
    </div>
    <fieldset className="settings-theme-choices" disabled={disabled}>
      <legend>{t("settings.featuredThemes")}</legend>
      <div className="settings-theme-featured">
        {featuredThemes.map(({ id, artwork, signature, name, description }) => <label key={id} className="settings-theme-card" data-choice={id}>
          <input type="radio" name="settings-theme" value={id} checked={selected === id} onChange={() => onChange(id)} />
          <span className="settings-theme-card__art" aria-hidden="true" style={{ backgroundImage: `url("${artwork}")` }}>
            <span className="settings-theme-card__mark"><Check size={15} /></span>
            <span className="settings-theme-card__signature">{signature}</span>
          </span>
          <span className="settings-theme-card__copy"><strong>{t(name)}</strong>
            <small>{t(description)}</small></span>
        </label>)}
      </div>
      <div className="settings-theme-choices__heading">{t("settings.simpleThemes")}</div>
      <div className="settings-theme-simple">
        {(["default", "ocean", "violet", "forest"] as const).map((id) => {
          const palette = deriveSettingsTheme(id, themes);
          return <label key={id} className="settings-theme-swatch" style={{ "--choice-bg": palette["--management-bg"], "--choice-accent": palette["--management-accent"] } as CSSProperties}>
            <input type="radio" name="settings-theme" value={id} checked={selected === id} onChange={() => onChange(id)} />
            <span className="settings-theme-swatch__sample" aria-hidden="true"><span /><span /><span /></span>
            <span>{t(`theme.${id}`)}</span><Check size={14} className="settings-theme-swatch__check" aria-hidden="true" />
          </label>;
        })}
      </div>
      <label className="settings-theme-follow"><input type="radio" name="settings-theme" value="follow-window" checked={selected === "follow-window"} onChange={() => onChange("follow-window")} />
        <span><strong>{t("settings.themeFollow")}</strong><small>{t("settings.themeFollowHint")}</small></span>
      </label>
    </fieldset>
    <p className="settings-theme-save" role="status">{disabled ? t("settings.saving") : t("settings.autoSave")}</p>
  </section>;
}
