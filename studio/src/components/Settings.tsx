import { X, FlipHorizontal, FlipVertical } from "lucide-react";
import { DisplaySettings, ROTATIONS } from "../hooks/useSettings.ts";

interface SettingsProps {
    settings: DisplaySettings;
    onChange: (patch: Partial<DisplaySettings>) => void;
    onReset: () => void;
    onClose: () => void;
}

const isDefault = (s: DisplaySettings) =>
    s.rotation === 0 && !s.flipHorizontal && !s.flipVertical;

function describe(s: DisplaySettings): string {
    const parts: string[] = [];
    if (s.rotation !== 0) parts.push(`Rotated ${s.rotation}° clockwise`);
    if (s.flipHorizontal) parts.push("mirrored horizontally");
    if (s.flipVertical) parts.push("mirrored vertically");
    if (parts.length === 0) return "Sent exactly as drawn";
    const text = parts.join(", ");
    return text.charAt(0).toUpperCase() + text.slice(1);
}

function OrientationPreview({ settings }: { settings: DisplaySettings }) {
    const sx = settings.flipHorizontal ? -1 : 1;
    const sy = settings.flipVertical ? -1 : 1;
    return (
        <div className="settings-preview">
            <svg
                viewBox="0 0 32 32"
                className="settings-preview-glyph"
                style={{ transform: `scale(${sx}, ${sy}) rotate(${settings.rotation}deg)` }}
                aria-hidden="true"
            >
                <rect x="1" y="1" width="30" height="30" rx="3" className="settings-preview-frame" />
                <rect x="10" y="8" width="4" height="17" className="settings-preview-letter" />
                <rect x="10" y="8" width="13" height="4" className="settings-preview-letter" />
                <rect x="10" y="15" width="9" height="4" className="settings-preview-letter" />
                <circle cx="5.5" cy="5.5" r="2" className="settings-preview-origin" />
            </svg>
            <div className="settings-preview-text">
                <span className="settings-preview-caption">What the matrix receives</span>
                <span className="settings-preview-summary">{describe(settings)}</span>
            </div>
        </div>
    );
}

interface SwitchRowProps {
    label: string;
    checked: boolean;
    onToggle: () => void;
    icon: React.ReactNode;
}

function SwitchRow({ label, checked, onToggle, icon }: SwitchRowProps) {
    return (
        <button
            type="button"
            role="switch"
            aria-checked={checked}
            className={`settings-switch-row ${checked ? "is-on" : ""}`}
            onClick={onToggle}
        >
            <span className="settings-switch-icon">{icon}</span>
            <span className="settings-switch-label">{label}</span>
            <span className="settings-switch-state">{checked ? "On" : "Off"}</span>
            <span className="settings-switch-track">
                <span className="settings-switch-thumb" />
            </span>
        </button>
    );
}

export default function Settings({ settings, onChange, onReset, onClose }: SettingsProps) {

    return (
        <div className="settings-pop-up-container" role="dialog" aria-label="Display settings">
            <div className="settings-header">
                <div>
                    <div className="settings-title">Display orientation</div>
                    <div className="settings-hint">Match how your matrix is mounted. Your drawings are not changed.</div>
                </div>
                <button className="icon-button" onClick={onClose} title="Close">
                    <X className="ic-btn" />
                </button>
            </div>

            <OrientationPreview settings={settings} />

            <div className="settings-group">
                <div className="settings-group-label">Rotation</div>
                <div className="settings-segmented" role="radiogroup" aria-label="Rotation">
                    {ROTATIONS.map((deg) => {
                        const selected = settings.rotation === deg;
                        return (
                            <button
                                key={deg}
                                type="button"
                                role="radio"
                                aria-checked={selected}
                                className={`settings-segment ${selected ? "is-selected" : ""}`}
                                onClick={() => onChange({ rotation: deg })}
                            >
                                {deg}°
                            </button>
                        );
                    })}
                </div>
            </div>

            <div className="settings-group">
                <div className="settings-group-label">Mirror</div>
                <SwitchRow
                    label="Horizontal"
                    icon={<FlipHorizontal className="ic-btn" />}
                    checked={settings.flipHorizontal}
                    onToggle={() => onChange({ flipHorizontal: !settings.flipHorizontal })}
                />
                <SwitchRow
                    label="Vertical"
                    icon={<FlipVertical className="ic-btn" />}
                    checked={settings.flipVertical}
                    onToggle={() => onChange({ flipVertical: !settings.flipVertical })}
                />
            </div>

            <button
                type="button"
                className="settings-reset"
                onClick={onReset}
                disabled={isDefault(settings)}
            >
                Reset to defaults
            </button>
        </div>
    );
}