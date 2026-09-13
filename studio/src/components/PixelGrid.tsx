import {RGB} from "../types.ts";

interface PixelGridProps {
    layout: RGB[];
    onMouseDown: (index: number) => void;
    onMouseEnter: (index: number) => void;
}

export default function PixelGrid({ layout, onMouseDown, onMouseEnter }: PixelGridProps) {
    return (
        <div className="pixel-grid-container">
            <div className="pixel-grid">
                {layout.map((pixel, i) => (
                    <div
                        key={i}
                        className="pixel-grid-item"
                        style={{ backgroundColor: `rgb(${pixel.r}, ${pixel.g}, ${pixel.b})` }}
                        onMouseDown={() => onMouseDown(i)}
                        onMouseEnter={() => onMouseEnter(i)}
                    />
                ))}
            </div>
        </div>
    );
}