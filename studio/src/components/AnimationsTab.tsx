import {Plus, Trash, Pencil} from "lucide-react";
import {useEffect, useState} from "react";
import AnimationFrame from "./AnimationFrame.tsx";
import {Animation} from "../hooks/useAnimationLibrary.ts";
import {RGB} from "../types.ts";

interface AnimationsTabProps {
    frames: RGB[][];
    onChangeFps: (id: string, fps: number) => void;
    activeFrameIndex: number;
    onAddFrame: () => void;
    onChangeFrame: (index: number) => void;
    onDeleteFrame: () => void;

    animations: Animation[];
    activeAnimationId: string | null;
    onSelectAnimation: (id: string) => void;
    onNewAnimation: () => void;
    onRenameAnimation: (id: string, name: string) => void;
    onDeleteAnimation: (id: string) => void;

}

export default function AnimationsTab({frames, activeFrameIndex, onAddFrame, onChangeFrame, onDeleteFrame, animations, activeAnimationId, onSelectAnimation, onNewAnimation, onRenameAnimation, onDeleteAnimation, onChangeFps}: AnimationsTabProps) {
    const [isEditingName, setIsEditingName] = useState(false);
    const [draftName, setDraftName] = useState("");

    const activeAnimation = animations.find((a) => a.id === activeAnimationId) ?? null;

    const [draftFps, setDraftFps] = useState<number>(activeAnimation?.fps ?? 1);


    const startEditingName = () => {
        if (!activeAnimation) return;
        setDraftName(activeAnimation.name);
        setIsEditingName(true);
    };

    const commitAnimationRename = () => {
        if (activeAnimation && draftName.trim()) {
            onRenameAnimation(activeAnimation.id, draftName.trim());
        }
        setIsEditingName(false);
    };

    const commitFpsChange = (value : number) => {
        if(activeAnimation){
            onChangeFps(activeAnimation.id, value);
        }
    }

    useEffect(() => {
        setDraftFps(activeAnimation?.fps ?? 1);
    },[activeAnimation?.id, activeAnimation?.fps]);

    return (
        <div className="animations-container">
            <div className="animation-library-header">
                <select
                    className="animation-select"
                    value={activeAnimationId ?? ""}
                    onChange={(e) => onSelectAnimation(e.target.value)}
                >
                    {animations.length === 0 && <option value="">No animations yet</option>}
                    {animations.map((animation) => (
                        <option key={animation.id} value={animation.id}>
                            {animation.name}
                        </option>
                    ))}
                </select>
                <button className="icon-button" title="New animation" onClick={onNewAnimation}>
                    <Plus className="ic-btn" />
                </button>
                <button
                    className="icon-button"
                    title="Delete animation"
                    disabled={!activeAnimation || animations.length <= 1}
                    onClick={() => activeAnimation && onDeleteAnimation(activeAnimation.id)}
                >
                    <Trash className="ic-btn" />
                </button>
            </div>

            {activeAnimation && (
                <>
                <div className="animation-name-row">
                    {isEditingName ? (
                        <input
                            className="animation-name-input"
                            value={draftName}
                            autoFocus
                            onChange={(e) => setDraftName(e.currentTarget.value)}
                            onBlur={commitAnimationRename}
                            onKeyDown={(e) => e.key === "Enter" && commitAnimationRename()}
                        />
                    ) : (
                        <button className="animation-name-btn" onClick={startEditingName}>
                            {activeAnimation.name}
                            <Pencil className="ic-btn-small" />
                        </button>
                    )}
                </div>
                    <div className="animation-name-row">
                        <input type="range" onChange={(e) => commitFpsChange(Number(e.target.value))} value={draftFps} min={1} max={60} />
                    </div>
                </>
            )}

            <div className="frames-list" >
                {frames.map((frame, index) => (
                    <AnimationFrame
                        key={index}
                        layout={frame}
                        isActive={activeFrameIndex === index}
                        onClick={() => onChangeFrame(index)}
                        index={index}
                    />
                ))}
            </div>
            <div className="action-btn-container">
                <button className="animation-btn" onClick={onAddFrame} title="Insert frame after current">
                    <Plus className="ic-btn"/>
                </button>
                <button
                    className="animation-btn"
                    onClick={onDeleteFrame}
                    disabled={frames.length <= 1}
                    title="Delete current frame"
                >
                    <Trash className="ic-btn"/>
                </button>
            </div>
        </div>
    )
}