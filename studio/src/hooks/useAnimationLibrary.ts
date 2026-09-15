import { useEffect, useState } from "react";
import { RGB } from "../types";

export interface Animation {
    id: string;
    name: string;
    frames: RGB[][];
}

const STORAGE_KEY = "pixelart-animations";

function loadFromStorage(): Animation[] {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw) return [];
        const parsed = JSON.parse(raw);
        return Array.isArray(parsed) ? parsed : [];
    } catch (e) {
        console.error("Failed to load saved animations:", e);
        return [];
    }
}

function persistToStorage(animations: Animation[]) {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(animations));
    } catch (e) {
        console.error("Failed to save animations:", e);
    }
}

export function useAnimationLibrary() {
    const [animations, setAnimations] = useState<Animation[]>(loadFromStorage);
    const [activeAnimationId, setActiveAnimationId] = useState<string | null>(
        () => loadFromStorage()[0]?.id ?? null
    );



    useEffect(() => {
        persistToStorage(animations);
    }, [animations]);

    const activeAnimation = animations.find((a) => a.id === activeAnimationId) ?? null;

    const createAnimation = (name: string, frames: RGB[][]) => {
        const animation: Animation = { id: crypto.randomUUID(), name, frames };
        setAnimations((prev) => [...prev, animation]);
        setActiveAnimationId(animation.id);
        return animation;
    };

    const updateActiveAnimationFrames = (frames: RGB[][]) => {
        if (!activeAnimationId) return;
        setAnimations((prev) =>
            prev.map((a) => (a.id === activeAnimationId ? { ...a, frames } : a))
        );
    };

    const renameAnimation = (id: string, name: string) => {
        setAnimations((prev) => prev.map((a) => (a.id === id ? { ...a, name } : a)));
    };

    const deleteAnimation = (id: string) => {
        setAnimations((prev) => prev.filter((a) => a.id !== id));
        setActiveAnimationId((prevId) => {
            if (prevId !== id) return prevId;
            const remaining = animations.filter((a) => a.id !== id);
            return remaining[0]?.id ?? null;
        });
    };

    const selectAnimation = (id: string) => {
        setActiveAnimationId(id);
    };

    return {
        animations,
        activeAnimation,
        activeAnimationId,
        createAnimation,
        updateActiveAnimationFrames,
        renameAnimation,
        deleteAnimation,
        selectAnimation,
    };
}