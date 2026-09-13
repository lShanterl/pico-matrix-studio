import { useState, useRef } from "react";
import {RGB} from "../types";

const SIZE = 16;
const PIXEL_COUNT = SIZE * SIZE;
const DEFAULT_RGB: RGB = { r: 0, g: 0, b: 0 };

export function usePixelLayout() {
    const [layout, setLayout] = useState<RGB[]>(
        Array.from({ length: PIXEL_COUNT }, () => DEFAULT_RGB)
    );
    const [operations, setOperations] = useState<RGB[][]>([]);
    const [redoHistory, setRedoHistory] = useState<RGB[][]>([]);
    const layoutRef = useRef<RGB[]>(layout);
    layoutRef.current = layout;

    const colorsMatch = (c1: RGB, c2: RGB) =>
        c1.r === c2.r && c1.g === c2.g && c1.b === c2.b;

    const drawPixel = (index: number, color: RGB) => {
        setLayout((prev) => {
            const next = [...prev];
            next[index] = color;
            return next;
        });
    };

    const erasePixel = (index: number) => drawPixel(index, DEFAULT_RGB);

    const clear = () => {
        setOperations((prev) => [...prev, layoutRef.current]);
        setRedoHistory([]);
        setLayout(Array.from({ length: PIXEL_COUNT }, () => DEFAULT_RGB));
    };

    const bucketFill = (startIndex: number, fillColor: RGB) => {
        setLayout((prev) => {
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
    // simple undo & redo, for a 16x16 frame adding batching would be an overkill
    const undo = () => {
        setOperations((prev) => {
            if (prev.length === 0) return prev;
            const next = [...prev];
            const lastState = next.pop()!;
            setRedoHistory((r) => [...r, layoutRef.current]);
            setLayout(lastState);
            return next;
        });
    };

    const redo = () => {
        setRedoHistory((prev) => {
            if (prev.length === 0) return prev;
            const next = [...prev];
            const nextState = next.pop()!;
            setOperations((o) => [...o, layoutRef.current]);
            setLayout(nextState);
            return next;
        });
    };

    const commitStroke = (before: RGB[]) => {
        if (JSON.stringify(before) !== JSON.stringify(layoutRef.current)) {
            setOperations((prev) => [...prev, before]);
            setRedoHistory([]);
        }
    };

    return {
        layout,
        drawPixel,
        erasePixel,
        clear,
        bucketFill,
        undo,
        redo,
        canUndo: operations.length > 0,
        canRedo: redoHistory.length > 0,
        commitStroke
    };
}