import {Eraser, PaintBucket, Pencil, Pipette, Play, Redo, Trash, Undo} from "lucide-react";
import {Tools} from "../types.ts";

interface FloatingToolbarProps {
    connected: boolean;
    onSend: () => void;
    activeTool: Tools;
    onToolChange: (tool: Tools) => void;
    onUndo: () => void;
    onRedo: () => void;
    canUndo: boolean;
    canRedo: boolean;
    onClear: () => void;
}

export default function FloatingToolbar({
        connected,
        onSend,
        activeTool,
        onToolChange,
        onUndo,
        onRedo,
        canUndo,
        canRedo,
        onClear,
    }: FloatingToolbarProps)
{
    return(
    <div className="floating-toolbar">
        <button
            className={`floating-toolbar-btn action-btn`}
            disabled={!connected}
            title="Send Frame to Pico"
            onClick={onSend}
        >
            <Play className="ic-btn" />
        </button>
        <div className="toolbar-divider"></div>
        <button
            className={`floating-toolbar-btn ${activeTool === Tools.Pencil ? "active" : ""}`}
            onClick={() => onToolChange(Tools.Pencil)}
        >
            <Pencil className="ic-btn" />
        </button>
        <button
            className={`floating-toolbar-btn ${activeTool === Tools.Eraser ? "active" : ""}`}
            onClick={() => onToolChange(Tools.Eraser)}
        >
            <Eraser className="ic-btn" />
        </button>
        <button
            className={`floating-toolbar-btn ${activeTool === Tools.Pipette ? "active" : ""}`}
            onClick={() => onToolChange(Tools.Pipette)}
        >
            <Pipette className="ic-btn" />
        </button>
        <button
            className={`floating-toolbar-btn ${activeTool === Tools.Bucket ? "active" : ""}`}
            onClick={() => onToolChange(Tools.Bucket)}
        >
            <PaintBucket className="ic-btn" />
        </button>
        <div className="toolbar-divider"></div>
        <button
            className="floating-toolbar-btn"
            onClick={onUndo}
            disabled={!canUndo}
        >
            <Undo className="ic-btn" />
        </button>
        <button
            className="floating-toolbar-btn"
            onClick={onRedo}
            disabled={!canRedo}
        >
            <Redo className="ic-btn" />
        </button>
        <button
            className="floating-toolbar-btn action-btn-trash"
            onClick={onClear}
        >
            <Trash className="ic-btn" />
        </button>
    </div>
    );
}