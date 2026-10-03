import { X, FlipHorizontal, FlipVertical } from "lucide-react";
import { DisplaySettings, ROTATIONS } from "../hooks/useSettings.ts";
import {useCallback, useEffect, useRef, useState} from "react";

interface SettingsProps {
    settings: DisplaySettings;
    onChange: (patch: Partial<DisplaySettings>) => void;
    onReset: () => void;
    onClose: () => void;
    onSetMaxCurrent: (current: number) => Promise<void>;
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

export default function Settings({ settings, onChange, onReset, onClose, onSetMaxCurrent }: SettingsProps) {
    const [isHolding, setIsHolding] = useState(false);
    const [progress, setProgress] = useState(0);
    const [isApplied, setIsApplied] = useState(false);

    const [sliderCurrent, setSliderCurrent] = useState(settings.currentMa);

    const intervalStep = 20;
    const timeToConfirm = 3000;
    const timerRef = useRef<number | null> (null);
    const intervalRef = useRef<number | null> (null);

    const clearPress = useCallback(() => {
        setIsHolding(false);
        if (timerRef.current) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }
        if (intervalRef.current) {
            clearInterval(intervalRef.current);
            intervalRef.current = null;
        }
    }, []);
    // keeep slider in sync if the current is changed from outside

    useEffect(() => {
        setSliderCurrent(settings.currentMa);
    }, [settings.currentMa]);

    const startPress = useCallback(() => {
        setIsHolding(true);
        setProgress(0);
        const increment = (intervalStep / timeToConfirm) * 100;

        intervalRef.current = window.setInterval(() => {
            setProgress((prev) => {
                const next = prev + increment;
                if (next >= 100) {
                    if (intervalRef.current) clearInterval(intervalRef.current);
                    return 100;
                }
                return next;
            });
        }, intervalStep);

        timerRef.current = window.setTimeout(() => {
            setProgress(0);
            setIsApplied(true);

            onChange({ currentMa: sliderCurrent });
            onSetMaxCurrent(sliderCurrent);

            setTimeout(() => setIsApplied(false), 2500);
        }, timeToConfirm);
    }, [timeToConfirm, sliderCurrent, onChange, onSetMaxCurrent]);

    const progressBarStyle = (isHolding && progress > 0) ? {
        backgroundImage: `linear-gradient(var(--accent), var(--accent))`,
        backgroundSize: `${progress}% 100%`,
        backgroundRepeat: 'no-repeat',
        backgroundPosition: 'left center'
    } : undefined;

    return (
        <div className="settings-pop-up-container" >
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
                <div className="settings-segmented">
                    {ROTATIONS.map((deg) => {
                        const selected = settings.rotation === deg;
                        return (
                            <button
                                key={deg}
                                type="button"
                                role="radio"
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

            <div className="settings-group">
                <div className="settings-header">
                    <div>
                        <div className="settings-title">Current limit</div>
                        <div className="settings-hint">Set the maximum current your power supply can deliver. Values above its rating may overheat and damage the supply, and possibly the matrix. </div>
                    </div>
                </div>
                <div className="settings-current-group">
                    <div className="setting-current">{sliderCurrent}A</div>
                    <input type="range" max={20} min={0.3} step={0.1} className="settings-current-slider" value={sliderCurrent} onChange={(e) => setSliderCurrent(Number(e.target.value))}/>
                </div>

            </div>

            <div className="settings-confirm-button-group">
                <button
                    type="button"
                    className="settings-reset"
                    onClick={onReset}
                    disabled={isDefault(settings)}
                >
                    Reset to defaults
                </button>
                <button
                    type="button"
                    className={`settings-reset apply-current-btn ${isHolding ? "is-active" : ""}`}
                    onMouseDown={startPress}
                    onMouseUp={clearPress}
                    onMouseLeave={clearPress}
                    disabled={isDefault(settings)}
                    style={progressBarStyle}
                >
                    {isApplied ? `Applied ${sliderCurrent}A`: `Hold to apply ${sliderCurrent}A`}
                </button>
            </div>
        </div>
    );
}