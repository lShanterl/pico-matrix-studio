import {RGB} from "../types.ts";

interface PixelGridProps {
    layout: RGB[];
    onMouseDown: (index: number, e: React.MouseEvent<HTMLDivElement>) => void;
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
                        onMouseDown={(e) => onMouseDown(i, e)}
                        onMouseEnter={() => onMouseEnter(i)}
                    />
                ))}
            </div>
        </div>
    );
}