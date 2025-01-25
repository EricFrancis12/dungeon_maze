import { useState } from "react";
import { WorldStructureSchema } from "../lib/schemas";
import { acceptFileUpload, downloadAsJsonFile, fillToLength, safeSchemaParseJSON } from "../lib/utils";
import { WorldStructure, newCell, newWorldStructure } from "../lib/types";
import WorldStructureEditor from "../components/WorldStructureEditor";

export default function WorldStructureEditorPage() {
    const [worldStructure, setWorldStructure] = useState<WorldStructure>(newWorldStructure());
    const [fileName, setFileName] = useState("");

    async function handleLoadFromFile() {
        const file = await acceptFileUpload();
        if (!file) return;

        setFileName(file.name);
        const s = await file.text();

        const ws = await safeSchemaParseJSON(s, WorldStructureSchema);
        if (!ws) {
            console.error("error parsing file");
            return;
        }

        for (const chunk of ws.chunks) {
            if (chunk.cells.length < 4) {
                fillToLength(chunk.cells, () => [], 4);

                for (const row of chunk.cells) {
                    if (row.length < 4) {
                        fillToLength(row, newCell, 4);
                    }
                }
            }
        }

        setWorldStructure(ws);
    }

    function handleSave() {
        downloadAsJsonFile(worldStructure, fileName);
    }

    return (
        <main className="relative flex flex-col h-screen w-full">
            <div className="flex items-center gap-2 w-full p-2">
                <button onClick={handleLoadFromFile}>Load From File</button>
                <button onClick={handleSave}>Save</button>
            </div>
            <div className="flex-1 h-screen w-full p-4">
                <WorldStructureEditor
                    worldStructure={worldStructure}
                    setWorldStructure={setWorldStructure}
                />
            </div>
        </main>
    );
}