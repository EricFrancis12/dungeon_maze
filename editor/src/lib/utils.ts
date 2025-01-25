import { z } from "zod";
import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
    return twMerge(clsx(inputs));
}

export function fillToLength<T>(arr: T[], t: () => T, length: number) {
    if (arr.length >= length) {
        return arr;
    }

    while (arr.length < length) {
        arr.push(t());
    }
}

export function safeParseJSON(s: string): unknown | null {
    try {
        return JSON.parse(s);
    } catch (_) {
        return null;
    }
}

export async function safeSchemaParseJSON<T>(s: string, schema: z.ZodType<T>): Promise<T | null> {
    const { data, success } = await schema.spa(safeParseJSON(s));
    return success ? data : null;
}

export async function acceptFileUpload(): Promise<File | null> {
    const inputEle = document.createElement("input");
    inputEle.type = "file";

    return new Promise((resolve) => {
        const handleChange = (e: Event) => {
            const { files } = e.target as HTMLInputElement;

            inputEle.removeEventListener("change", handleChange);
            inputEle.remove();

            resolve(files?.[0] ?? null);
        };

        inputEle.addEventListener("change", handleChange);
        inputEle.click();
    });
}

export function downloadAsJsonFile<T>(data: T, fileName: string) {
    const content = JSON.stringify(data, null, 4);
    const blob = new Blob([content], { type: "application/json" });

    const aEle = document.createElement("a");
    aEle.download = fileName;
    aEle.href = URL.createObjectURL(blob);

    document.documentElement.appendChild(aEle);
    aEle.click();
    aEle.remove();
}
