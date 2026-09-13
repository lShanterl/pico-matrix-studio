
export interface RGB {
    r: number;
    g: number;
    b: number;
}

export interface ConnectionStatus {
    connected: boolean;
    ip: string;
}
export const IP = "192.168.1.50";

export const DEFAULT_RGB: RGB = { r: 0, g: 0, b: 0 };
export const ACTIVE_COLOR: RGB = { r: 239, g: 68, b: 68 };

export const builtInColors: RGB[] = [
    { r: 255, g: 59, b: 92 },
    { r: 255, g: 176, b: 32 },
    { r: 79, g: 209, b: 197 },
    { r: 91, g: 141, b: 239 },
    { r: 61, g: 220, b: 132 },
    { r: 194, g: 91, b: 222 },
];

export const sidebarTools: string[] = ["Draw", "Animations"];

export const enum MenuItems {
    Draw,
    Animations,
}

export const enum Tools {
    Pencil,
    Eraser,
    Pipette,
    Bucket,
}

export interface PowerEstimate {
    currentMa: number;
    maxCurrentMa: number;
    overLimit: number;
}
