import {useRef, useState} from "react";
import "./App.css";
import {Settings} from "lucide-react";
import {invoke} from "@tauri-apps/api/core";
import {usePicoConnection} from "./hooks/usePicoConnection.ts";
import {
    ACTIVE_COLOR,
    IP,
    MenuItems,
    RGB, sidebarTools,
    Tools
} from "./types.ts";
import {usePixelLayout} from "./hooks/usePixelLayout.ts";
import FloatingToolbar from "./components/FloatingToolbar.tsx";
import PixelGrid from "./components/PixelGrid.tsx";
import {usePowerEstimate} from "./hooks/usePowerEstimate.ts";
import ColorTools from "./components/ColorTools.tsx";


export default function App() {

    const pixels = usePixelLayout();
    const { status, connect, disconnect } = usePicoConnection();
    const power = usePowerEstimate(pixels.layout);

    const [isDrawing, setDrawing] = useState<boolean>(false);
    const initialMatrixRef = useRef<RGB[]>([]);

    const [activeColor, setActiveColor] = useState<RGB>(ACTIVE_COLOR);
    const [activeMenu, setActiveMenu] = useState<MenuItems>(MenuItems.Draw);
    const [areSettingsOpen, setAreSettingsOpen] = useState(false);
    const [activeTool, setActiveTool] = useState<Tools>(Tools.Pencil);



    const handleConnectClick = async () =>{
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

    const handleMouseDown = (index: number): void => {
        initialMatrixRef.current = [...pixels.layout];

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
        pixels.commitStroke(initialMatrixRef.current);
        window.removeEventListener("mouseup", handleMouseUp);
    }

    const useTool = (index: number) => {
        switch (activeTool) {
            case Tools.Pencil: pixels.drawPixel(index, activeColor); break;
            case Tools.Eraser: pixels.erasePixel(index); break;
            case Tools.Bucket: pixels.bucketFill(index, activeColor); break;
            case Tools.Pipette: setActiveColor(pixels.layout[index]); break;
        }
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
                        <Settings className="ic-btn" />
                    </button>
                </div>

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
                    <PixelGrid
                        layout={pixels.layout}
                        onMouseDown={handleMouseDown}
                        onMouseEnter={handleMouseEnter}
                    />
                    <FloatingToolbar
                        connected={status.connected}
                        onSend={() => invoke("send_frame_to_pico", { layout: pixels.layout })}
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