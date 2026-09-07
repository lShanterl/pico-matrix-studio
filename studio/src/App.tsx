import { useState } from "react";
import "./App.css";
import { Play, Pencil, Eraser, Undo, Redo, Pipette, Settings, PaintBucket } from "lucide-react";

const SIZE = 16;
const PIXEL_COUNT = SIZE * SIZE;

export interface RGB {
    r: number;
    g: number;
    b: number;
}

const DEFAULT_RGB: RGB = { r: 0, g: 0, b: 0 };
const ACTIVE_COLOR: RGB = { r: 239, g: 68, b: 68 };

const builtInColors: RGB[] = [
    { r: 255, g: 59, b: 92 },
    { r: 255, g: 176, b: 32 },
    { r: 79, g: 209, b: 197 },
    { r: 91, g: 141, b: 239 },
    { r: 61, g: 220, b: 132 },
    { r: 194, g: 91, b: 222 },
];

const sidebarTools: string[] = ["Draw", "Animations"];

const enum MenuItems {
    Draw,
    Solid,
    Animations,
}

const enum Tools {
    Pencil,
    Eraser,
    Pipette,
    Bucket,
}

export default function App() {
    const [layout, setLayout] = useState<RGB[]>(Array.from({ length: PIXEL_COUNT }, (): RGB => DEFAULT_RGB));
    const [isDrawing, setDrawing] = useState<boolean>(false);
    const [activeColor, setActiveColor] = useState<RGB>(ACTIVE_COLOR);
    const [activeMenu, setActiveMenu] = useState<MenuItems>(MenuItems.Draw);
    const [areSettingsOpen, setAreSettingsOpen] = useState(false);
    const [activeTool, setActiveTool] = useState<Tools>(Tools.Pencil);

    const [initialMatrix, setInitialMatrix] = useState<RGB[]>([]);
    const [operations, setOperations] = useState<RGB[][]>([]);
    const [redoHistory, setRedoHistory] = useState<RGB[][]>([]);

    const colorsMatch = (c1: RGB, c2: RGB): boolean => {
        return c1.r === c2.r && c1.g === c2.g && c1.b === c2.b;
    };

    const drawPixel = (index: number, color?: RGB): void => {
        const targetColor = color ? color : activeColor;
        setLayout((prevLayout) => {
            const next = [...prevLayout];
            next[index] = targetColor;
            return next;
        });
    };

    const erasePixel = (index: number): void => {
        drawPixel(index, DEFAULT_RGB);
    };

    const getPixelColor = (index: number): RGB => {
        return layout[index];
    };

    const bucketFill = (startIndex: number, targetColor?: RGB): void => {
        const fillColor = targetColor ? targetColor : activeColor;
        const startColor = layout[startIndex];

        if (colorsMatch(startColor, fillColor)) return;

        const nextLayout = [...layout];
        const queue: number[] = [startIndex];
        const visited = new Set<number>();

        while (queue.length > 0) {
            const current = queue.pop()!;
            if (visited.has(current)) continue;
            visited.add(current);

            if (colorsMatch(nextLayout[current], startColor)) {
                nextLayout[current] = fillColor;

                const x = current % SIZE;
                const y = Math.floor(current / SIZE);

                if (x > 0) queue.push(current - 1);
                if (x < SIZE - 1) queue.push(current + 1);
                if (y > 0) queue.push(current - SIZE);
                if (y < SIZE - 1) queue.push(current + SIZE);
            }
        }
        setLayout(nextLayout);
    };

    // simple undo & redo, for a 16x16 frame adding batching would be an overkill
    const undo = (): void => {
        if (operations.length > 0) {
            const previousOperations = [...operations];
            const lastState = previousOperations.pop();
            if (lastState) {
                setRedoHistory((prev) => [...prev, layout]);
                setLayout(lastState);
                setOperations(previousOperations);
            }
        }
    };

    const redo = (): void => {
        if (redoHistory.length > 0) {
            const previousRedo = [...redoHistory];
            const nextState = previousRedo.pop();
            if (nextState) {
                setOperations((prev) => [...prev, layout]);
                setLayout(nextState);
                setRedoHistory(previousRedo);
            }
        }
    };

    const handleMouseDown = (index: number): void => {
        setInitialMatrix([...layout]);
        setDrawing(true);
        useTool(index);

        addEventListener("mouseup", handleMouseUp);
    };

    const handleMouseEnter = (index: number): void => {
        if (isDrawing && (activeTool === Tools.Pencil || activeTool === Tools.Eraser)) {
            useTool(index);
        }
    };

    const handleMouseUp = (): void =>{
        setDrawing(false);
        if (JSON.stringify(initialMatrix) !== JSON.stringify(layout)) {
            setOperations((prev) => [...prev, initialMatrix]);
            setRedoHistory([]);
        }
        window.removeEventListener("mouseup", handleMouseUp);

    }


    const useTool = (index: number): void => {
        switch (activeTool) {
            case Tools.Pencil:
                drawPixel(index);
                break;
            case Tools.Eraser:
                erasePixel(index);
                break;
            case Tools.Pipette:
                setActiveColor(getPixelColor(index));
                break;
            case Tools.Bucket:
                bucketFill(index);
                break;
        }
    };

    const applySolidColor = (color: RGB): void => {
        setOperations((prev) => [...prev, layout]);
        setRedoHistory([]);
        setLayout(Array.from({ length: PIXEL_COUNT }, (): RGB => color));
    };

    const rgbToHex = (color: RGB): string => {
        const hex = ((color.r << 16) | (color.g << 8) | color.b).toString(16).padStart(6, "0");
        return `#${hex}`;
    };

    const hexToRgb = (color: string): RGB => {
        const cleanHex = color.replace("#", "");
        return {
            r: parseInt(cleanHex.slice(0, 2), 16) || 0,
            g: parseInt(cleanHex.slice(2, 4), 16) || 0,
            b: parseInt(cleanHex.slice(4, 6), 16) || 0,
        };
    };

    return (
        <div className="app">
            <div className="titlebar">
                <span className="titlebar-connection">Pico IP: 192.168.x.x</span>
                <button className="icon-button" onClick={() => setAreSettingsOpen(!areSettingsOpen)} title="Settings">
                    <Settings className="ic-btn" />
                </button>
            </div>

            {areSettingsOpen && (
                <div className="settings-pop-up-container">

                </div>
            )}

            <div className="app-body">
                <div className="sidebar">
                    <span className="sidebar-title">Menu</span>
                    <div className="sidebar-tools">
                        {sidebarTools.map((tool, index) => (
                            <button
                                className={`sidebar-tool-btn ${activeMenu === index ? "active" : ""}`}
                                key={tool}
                                onClick={() => {
                                    setActiveMenu(index);
                                }}
                            >
                                {tool}
                            </button>
                        ))}
                    </div>
                </div>

                <div className="workspace">
                    <div className="pixel-grid-container">
                        <div className="pixel-grid">
                            {layout.map((pixel, i) => (
                                <div
                                    key={i}
                                    className="pixel-grid-item"
                                    style={{ backgroundColor: `rgb(${pixel.r}, ${pixel.g}, ${pixel.b})` }}
                                    onMouseDown={() => handleMouseDown(i)}
                                    onMouseEnter={() => handleMouseEnter(i)}
                                ></div>
                            ))}
                        </div>
                    </div>

                    <div className="floating-toolbar">
                        <button className="floating-toolbar-btn action-btn" title="Send Frame to Pico">
                            <Play className="ic-btn" />
                        </button>
                        <div className="toolbar-divider"></div>
                        <button
                            className={`floating-toolbar-btn ${activeTool === Tools.Pencil ? "active" : ""}`}
                            onClick={() => setActiveTool(Tools.Pencil)}
                        >
                            <Pencil className="ic-btn" />
                        </button>
                        <button
                            className={`floating-toolbar-btn ${activeTool === Tools.Eraser ? "active" : ""}`}
                            onClick={() => setActiveTool(Tools.Eraser)}
                        >
                            <Eraser className="ic-btn" />
                        </button>
                        <button
                            className={`floating-toolbar-btn ${activeTool === Tools.Pipette ? "active" : ""}`}
                            onClick={() => setActiveTool(Tools.Pipette)}
                        >
                            <Pipette className="ic-btn" />
                        </button>
                        <button
                            className={`floating-toolbar-btn ${activeTool === Tools.Bucket ? "active" : ""}`}
                            onClick={() => setActiveTool(Tools.Bucket)}
                        >
                            <PaintBucket className="ic-btn" />
                        </button>
                        <div className="toolbar-divider"></div>
                        <button
                            className="floating-toolbar-btn"
                            onClick={undo}
                            disabled={operations.length === 0}
                        >
                            <Undo className="ic-btn" />
                        </button>
                        <button
                            className="floating-toolbar-btn"
                            onClick={redo}
                            disabled={redoHistory.length === 0}
                        >
                            <Redo className="ic-btn" />
                        </button>
                    </div>
                </div>

                <div className="transform-matrix-container">
                    <div className="color-tools-container">
                        <div className="color-picker-row">
                            <div className="color-picker-container">
                                <input
                                    id="colorPicker"
                                    className="color-input-overlay"
                                    type="color"
                                    value={rgbToHex(activeColor)}
                                    onChange={(e) => setActiveColor(hexToRgb(e.currentTarget.value))}
                                />
                                <label
                                    className="custom-color-btn"
                                    htmlFor="colorPicker"
                                    style={{ backgroundColor: `rgb(${activeColor.r}, ${activeColor.g}, ${activeColor.b})` }}
                                ></label>
                            </div>
                            <input
                                type="text"
                                className="hex-input"
                                value={rgbToHex(activeColor)}
                                onChange={(e) => {
                                    const val = e.currentTarget.value;
                                    if (/^#[0-9a-fA-F]{6}$/.test(val)) {
                                        setActiveColor(hexToRgb(val));
                                    }
                                }}
                            />
                        </div>

                        <div className="built-in-colors-container">
                            {builtInColors.map((color, index) => (
                                <button
                                    className="color-button"
                                    key={index}
                                    onClick={() => setActiveColor(color)}
                                    style={{ backgroundColor: `rgb(${color.r}, ${color.g}, ${color.b})` }}
                                />
                            ))}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}