import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {PowerEstimate, RGB} from "../types.ts";

const DEBOUNCE_MS = 100; // avoid hammering the backend while actively drawing

export function usePowerEstimate(layout: RGB[]): PowerEstimate | null {
    const [power, setPower] = useState<PowerEstimate | null>(null);

    useEffect(() => {
        let cancelled = false;

        const id = setTimeout(() => {
            invoke<PowerEstimate>("estimate_power", { layout })
                .then((result) => {
                    if (!cancelled) setPower(result);
                })
                .catch((e) => {
                    console.error("Failed to estimate power:", e);
                });
        }, DEBOUNCE_MS);

        return () => {
            cancelled = true;
            clearTimeout(id);
        };
    }, [layout]);

    return power;
}