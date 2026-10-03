import { useEffect, useState } from "react";
import { RGB, SETTINGS_KEY } from "../types.ts";

export type Rotation = 0 | 90 | 180 | 270;
export const ROTATIONS: Rotation[] = [0, 90, 180, 270];

export interface DisplaySettings {
    rotation: Rotation;
    flipHorizontal: boolean;
    flipVertical: boolean;
    currentMa: number;
}

const SIZE = 16;

export const DEFAULT_DISPLAY_SETTINGS: DisplaySettings = {
    rotation: 0,
    flipHorizontal: false,
    flipVertical: false,
    currentMa: 0.5,
};

function loadSettings(): DisplaySettings {
    try {
        const raw = localStorage.getItem(SETTINGS_KEY);
        if (!raw) return DEFAULT_DISPLAY_SETTINGS;
        const parsed = JSON.parse(raw);
        return {
            rotation: ROTATIONS.includes(parsed.rotation) ? parsed.rotation : 0,
            flipHorizontal: !!parsed.flipHorizontal,
            flipVertical: !!parsed.flipVertical,
            currentMa: typeof parsed.currentMa === "number" ? parsed.currentMa : DEFAULT_DISPLAY_SETTINGS.currentMa,
        };
    } catch {
        return DEFAULT_DISPLAY_SETTINGS;
    }
}

export function useDisplaySettings() {
    const [settings, setSettings] = useState<DisplaySettings>(loadSettings);

    useEffect(() => {
        try {
            localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
        } catch {

        }
    }, [settings]);

    const updateSettings = (patch: Partial<DisplaySettings>) =>
        setSettings((prev) => ({ ...prev, ...patch }));

    const resetSettings = () => setSettings(DEFAULT_DISPLAY_SETTINGS);

    return { settings, updateSettings, resetSettings };
}

export function applyDisplayTransform(
    frames: RGB[][],
    settings: DisplaySettings,
): RGB[][] {
    const last = SIZE - 1;

    return frames.map((frame) => {
        const out: RGB[] = new Array(SIZE * SIZE).fill({ r: 0, g: 0, b: 0 });

        for (let y = 0; y < SIZE; y++) {
            for (let x = 0; x < SIZE; x++) {
                let dx = x;
                let dy = y;

                switch (settings.rotation) {
                    case 90:  dx = last - y; dy = x; break;
                    case 180: dx = last - x; dy = last - y; break;
                    case 270: dx = y; dy = last - x; break;
                }

                if (settings.flipHorizontal) dx = last - dx;
                if (settings.flipVertical) dy = last - dy;

                const pixel = frame[y * SIZE + x];
                out[dy * SIZE + dx] = {
                    r: Math.round(pixel.r),
                    g: Math.round(pixel.g),
                    b: Math.round(pixel.b),
                };
            }
        }
        return out;
    });
}