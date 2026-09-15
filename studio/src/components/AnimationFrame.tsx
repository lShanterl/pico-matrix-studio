import {RGB} from "../types.ts";
import {useMemo} from "react";

interface AnimationFrameProps {
    layout: RGB[];
    isActive: boolean;
    onClick: () => void;
    index: number;
}

export default function AnimationFrame({ layout, isActive, index, onClick }: AnimationFrameProps){


    const thumbnailSrc = useMemo(() => {
        const size = Math.sqrt(layout.length);
        const canvas = document.createElement("canvas");
        canvas.width = size;
        canvas.height = size;
        const ctx = canvas.getContext("2d");

        if (ctx) {
            layout.forEach((pixel, index) => {
                const x = index % size;
                const y = Math.floor(index / size);
                ctx.fillStyle = `rgb(${pixel.r}, ${pixel.g}, ${pixel.b})`;
                ctx.fillRect(x, y, 1, 1);
            });
        }

        return canvas.toDataURL("image/png");
    }, [layout])

    return (
        <div
            className={`animation-btn ${isActive ? "active" : ""}`}
            onClick={onClick}
        >
            <img
                src={thumbnailSrc}
                className="animation-img"
                style={{
                }}
            />
            <div className="animation-index">{index}</div>
        </div>
    );
}