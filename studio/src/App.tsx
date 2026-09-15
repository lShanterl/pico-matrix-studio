import { useRef, useState, useEffect } from "react";
import "./App.css";
import { Settings as SettingsIcon } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { usePicoConnection } from "./hooks/usePicoConnection.ts";
import { ACTIVE_COLOR, IP, RGB, Tools } from "./types.ts";
import { usePixelLayout, createBlankFrame } from "./hooks/usePixelLayout.ts";
import { useAnimationLibrary } from "./hooks/useAnimationLibrary.ts";
import FloatingToolbar from "./components/FloatingToolbar.tsx";
import PixelGrid from "./components/PixelGrid.tsx";
import { usePowerEstimate } from "./hooks/usePowerEstimate.ts";
import ColorTools from "./components/ColorTools.tsx";
import Settings from "./components/Settings.tsx";
import AnimationsTab from "./components/AnimationsTab.tsx";

enum MouseActionType {
    Draw = 0,
    MiddleClick = 1,
    Erase= 2
}

export default function App() {
    const pixels = usePixelLayout();
    const library = useAnimationLibrary();
    const { status, connect, disconnect } = usePicoConnection();
    const power = usePowerEstimate(pixels.layout);

    const [isDrawing, setDrawing] = useState<boolean>(false);
    const initialMatrixRef = useRef<RGB[]>([]);

    const [activeColor, setActiveColor] = useState<RGB>(ACTIVE_COLOR);
    const [areSettingsOpen, setAreSettingsOpen] = useState(false);
    const [activeTool, setActiveTool] = useState<Tools>(Tools.Pencil);

    const mouseActionRef = useRef<MouseActionType | null>(null);

    // make sure there's always at least one animation to save into
    useEffect(() => {
        if (library.animations.length === 0) {
            library.createAnimation("Animation 1", pixels.frames);
        }
    }, []);

    useEffect(() => {
        if (library.activeAnimation) {
            pixels.loadFrames(library.activeAnimation.frames);
        }
    }, [library.activeAnimationId]);

    // save the animation with debounce
    useEffect(() => {
        const id = setTimeout(() => {
            library.updateActiveAnimationFrames(pixels.frames);
        }, 400);
        return () => clearTimeout(id);
    }, [pixels.frames]);

    const handleConnectClick = async () => {
        if (status.connected) {
            await disconnect();
        } else {
            try {
                await connect(IP);
            } catch (e) {
                console.error("Connection failed:", e);
            }
        }
    }

    const handleMouseDown = (index: number, e : React.MouseEvent<HTMLDivElement>): void => {
        initialMatrixRef.current = [...pixels.layout];
        setDrawing(true);

        mouseActionRef.current = e.button;

        useTool(index);

        addEventListener("mouseup", handleMouseUp);
    };

    const handleMouseEnter = (index: number): void => {

        if (isDrawing && (activeTool === Tools.Pencil || activeTool === Tools.Eraser)) {
            useTool(index);
        }
    };

    const handleMouseUp = (): void => {
        setDrawing(false);
        mouseActionRef.current = null;
        pixels.commitStroke(initialMatrixRef.current);
        window.removeEventListener("mouseup", handleMouseUp);
    }

    const useTool = (index: number) => {
        const color = mouseActionRef.current === MouseActionType.Draw ? activeColor : pixels.DEFAULT_RGB;
        switch (activeTool) {
            case Tools.Pencil: pixels.drawPixel(index, color); break;
            case Tools.Eraser: pixels.erasePixel(index); break;
            case Tools.Bucket: pixels.bucketFill(index, color); break;
            case Tools.Pipette: setActiveColor(pixels.layout[index]); break;
        }
    };

    const handleSelectAnimation = (id: string) => {
        library.selectAnimation(id);
    };

    const handleNewAnimation = () => {
        const name = `Animation ${library.animations.length + 1}`;
        library.createAnimation(name, [createBlankFrame()]);
    };

    const handleDeleteAnimation = (id: string) => {
        if (library.animations.length <= 1) return;
        if (!window.confirm("Delete this animation? This can't be undone.")) return;
        library.deleteAnimation(id);
    };

    return (
        <div className="app">
            <div className="titlebar">
                <span className="titlebar-connection">
                    <button className="icon-button" onClick={handleConnectClick}>
                        <span className={`status-dot ${status.connected ? "connected" : "disconnected"}`} />
                    </button>
                    <span>{status.ip}</span>
                </span>
                <div className="titlebar-right">
                    {power && (
                        <div className={`power-readout ${power.overLimit ? "over-limit" : ""}`}>
                            {Math.round(power.currentMa)} mA / {power.maxCurrentMa} mA
                        </div>
                    )}
                    <button className="icon-button" onClick={() => setAreSettingsOpen(!areSettingsOpen)} title="Settings">
                        <SettingsIcon className="ic-btn" />
                    </button>
                </div>
            </div>

            {areSettingsOpen && <Settings/>}

            <div className="app-body">
                <div className="sidebar">
                    <span className="sidebar-title">Menu</span>
                    <AnimationsTab
                        frames={pixels.frames}
                        onChangeFps={library.changeFps}
                        activeFrameIndex={pixels.activeFrameIndex}
                        onAddFrame={pixels.addFrame}
                        onChangeFrame={pixels.changeFrame}
                        onDeleteFrame={pixels.deleteFrame}
                        animations={library.animations}
                        activeAnimationId={library.activeAnimationId}
                        onSelectAnimation={handleSelectAnimation}
                        onNewAnimation={handleNewAnimation}
                        onRenameAnimation={library.renameAnimation}
                        onDeleteAnimation={handleDeleteAnimation}
                    />
                </div>

                <div className="workspace">
                    <PixelGrid
                        layout={pixels.layout}
                        onMouseDown={handleMouseDown}
                        onMouseEnter={handleMouseEnter}
                    />
                    <FloatingToolbar
                        connected={status.connected}
                        onSend={() => invoke("send_frame_to_pico", { frames: pixels.frames, fps: library.activeAnimation?.fps})}
                        activeTool={activeTool}
                        onToolChange={setActiveTool}
                        onUndo={pixels.undo}
                        onRedo={pixels.redo}
                        canUndo={pixels.canUndo}
                        canRedo={pixels.canRedo}
                        onClear={pixels.clear}
                    />
                </div>

                <div className="transform-matrix-container">
                    <ColorTools
                        activeColor={activeColor}
                        setActiveColor={setActiveColor}
                    />
                </div>
            </div>
        </div>
    );
}