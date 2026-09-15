import { useState, useRef } from "react";
import { RGB } from "../types";

const SIZE = 16;
const PIXEL_COUNT = SIZE * SIZE;
const DEFAULT_RGB: RGB = { r: 0, g: 0, b: 0 };

const blankLayout = (): RGB[] => Array.from({ length: PIXEL_COUNT }, () => DEFAULT_RGB);
const cloneLayout = (layout: RGB[]): RGB[] => layout.map((p) => ({ ...p }));

export const createBlankFrame = (): RGB[] => {
    return blankLayout();
};

const colorsMatch = (c1: RGB, c2: RGB) =>
    c1.r === c2.r && c1.g === c2.g && c1.b === c2.b;

export function usePixelLayout() {
    const [frames, setFrames] = useState<RGB[][]>([createBlankFrame()]);
    const [activeFrameIndex, setActiveFrameIndex] = useState(0);
    const [operations, setOperations] = useState<RGB[][]>([]);
    const [redoHistory, setRedoHistory] = useState<RGB[][]>([]);

    const layout = frames[activeFrameIndex] ?? blankLayout();
    const layoutRef = useRef<RGB[]>(layout);
    layoutRef.current = layout;

    const updateActiveLayout = (updater: (prev: RGB[]) => RGB[]) => {
        setFrames((prev) => {
            const current = prev[activeFrameIndex];
            if (!current) return prev;

            const nextLayout = updater(current);
            if (nextLayout === current) return prev;

            const next = [...prev];
            next[activeFrameIndex] = nextLayout;
            return next;
        });
    };

    const drawPixel = (index: number, color: RGB) => {
        updateActiveLayout((prev) => {
            const next = [...prev];
            next[index] = color;
            return next;
        });
    };

    const erasePixel = (index: number) => drawPixel(index, DEFAULT_RGB);

    const bucketFill = (startIndex: number, fillColor: RGB) => {
        updateActiveLayout((prev) => {
            const startColor = prev[startIndex];
            if (colorsMatch(startColor, fillColor)) return prev;
            const next = [...prev];
            const queue = [startIndex];
            const visited = new Set<number>();
            while (queue.length) {
                const current = queue.pop()!;
                if (visited.has(current)) continue;
                visited.add(current);
                if (colorsMatch(next[current], startColor)) {
                    next[current] = fillColor;
                    const x = current % SIZE, y = Math.floor(current / SIZE);
                    if (x > 0) queue.push(current - 1);
                    if (x < SIZE - 1) queue.push(current + 1);
                    if (y > 0) queue.push(current - SIZE);
                    if (y < SIZE - 1) queue.push(current + SIZE);
                }
            }
            return next;
        });
    };

    const clear = () => {
        setOperations((prev) => [...prev, layoutRef.current]);
        setRedoHistory([]);
        updateActiveLayout(() => blankLayout());
    };

    const commitStroke = (before: RGB[]) => {
        if (JSON.stringify(before) !== JSON.stringify(layoutRef.current)) {
            setOperations((prev) => [...prev, before]);
            setRedoHistory([]);
        }
    };

    const undo = () => {
        setOperations((prev) => {
            if (prev.length === 0) return prev;
            const next = [...prev];
            const lastState = next.pop()!;
            setRedoHistory((r) => [...r, layoutRef.current]);
            updateActiveLayout(() => lastState);
            return next;
        });
    };

    const redo = () => {
        setRedoHistory((prev) => {
            if (prev.length === 0) return prev;
            const next = [...prev];
            const nextState = next.pop()!;
            setOperations((o) => [...o, layoutRef.current]);
            updateActiveLayout(() => nextState);
            return next;
        });
    };

    const resetHistory = () => {
        setOperations([]);
        setRedoHistory([]);
    };

    const changeFrame = (index: number) => {
        if (index < 0 || index >= frames.length) return;
        setActiveFrameIndex(index);
        resetHistory();
    };

    const addFrame = () => {
        setFrames((prev) => {
            const current = prev[activeFrameIndex];
            const newFrame: RGB[] = current ? cloneLayout(current) : blankLayout();

            const insertAt = activeFrameIndex + 1;
            return [...prev.slice(0, insertAt), newFrame, ...prev.slice(insertAt)];
        });
        setActiveFrameIndex((prev) => prev + 1);
        resetHistory();
    };

    const deleteFrame = () => {
        if (frames.length <= 1) return;
        const newLength = frames.length - 1;
        setFrames((prev) => prev.filter((_, i) => i !== activeFrameIndex));
        setActiveFrameIndex((prev) => Math.min(prev, newLength - 1));
        resetHistory();
    };

    const loadFrames = (newFrames: RGB[][]) => {
        setFrames(newFrames.length > 0 ? newFrames : [createBlankFrame()]);
        setActiveFrameIndex(0);
        resetHistory();
    };

    return {
        layout,
        frames,
        activeFrameIndex,
        drawPixel,
        erasePixel,
        clear,
        bucketFill,
        undo,
        redo,
        canUndo: operations.length > 0,
        canRedo: redoHistory.length > 0,
        commitStroke,
        changeFrame,
        addFrame,
        deleteFrame,
        loadFrames,
        DEFAULT_RGB
    };
}