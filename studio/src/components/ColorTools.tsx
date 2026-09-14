import {DeleteIcon, Plus} from "lucide-react";
import {builtInColors, RGB} from "../types.ts";
import {useState} from "react";

interface ColorToolsProps{
    activeColor: RGB;
    setActiveColor: (color: RGB) => void;
}

export default function ColorTools(
    {activeColor, setActiveColor}: ColorToolsProps
){

    const [availableColors, setAvailableColors] = useState<RGB[]>([...builtInColors]);

    const colorsMatch = (a: RGB, b: RGB): boolean => {
        return a.r === b.r && b.g === b.g && b.b === b.b;
    }

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

    const handleAddColor = (): void => {
        const color = activeColor;
        if(availableColors.some((c) => colorsMatch(c, color))) return;
        setAvailableColors([...availableColors, color])
    }

    const handleRemoveColor = (): void => {
        const color = activeColor;
        const colors = availableColors.filter(c => !colorsMatch(c,color));
        setAvailableColors(colors);
    }

    return(
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
                {availableColors.map((color, index) => (
                    <button
                        className="color-button"
                        key={index}
                        onClick={() => setActiveColor(color)}
                        style={{ backgroundColor: `rgb(${color.r}, ${color.g}, ${color.b})` }}
                    />
                ))}
                <button className="icon-button" onClick={() => handleAddColor()}><Plus className="ic-btn"/> </button>
                <button className='icon-button' onClick={() => handleRemoveColor()}><DeleteIcon className='ic-btn'/></button>
            </div>
            <div className="sliders-container">
                <div className="slider">
                    <p>Brightness</p>
                    <input type="range"/>
                </div>
            </div>
        </div>
    );
}