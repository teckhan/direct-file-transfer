import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export function formatBytes(bytes?: number): string {
    if (bytes === undefined || Number.isNaN(bytes)) return "—";
    if (bytes < 1024) return `${bytes} B`;

    const units = ["KB", "MB", "GB", "TB"];
    let value = bytes;
    let unit = "B";
    for (const next of units) {
        if (value < 1024) break;
        value /= 1024;
        unit = next;
    }

    return `${value >= 10 ? Math.round(value) : value.toFixed(1)} ${unit}`;
}
