import { useState } from "react";
import { Button } from "../components/ui/Button";
import WorldStructureEditor from "../components/WorldStructureEditor";
import { WorldStructureSchema } from "../lib/schemas";
import { acceptFileUpload, downloadAsJsonFile, fillToLength, safeSchemaParseJSON, stripSuffixIfExists } from "../lib/utils";
import { WorldStructure, newCell, newWorldStructure } from "../lib/types";

export default function WorldStructureEditorPage() {
    const [worldStructure, setWorldStructure] = useState<WorldStructure>(newWorldStructure());
    const [name, setName] = useState("");

    async function handleLoadFromFile() {
        const file = await acceptFileUpload();
        if (!file) {
            console.error("error uploading file");
            return;
        };

        setName(stripSuffixIfExists(file.name, ".json"));
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
        downloadAsJsonFile(worldStructure, `${name}.json`);
    }

    return (
        <main className="relative flex flex-col h-screen w-full">
            <div className="flex items-center gap-2 w-full p-2">
                <Button onClick={handleLoadFromFile}>Load From File</Button>
                <Button onClick={handleSave}>Save</Button>
            </div>
            <div className="flex-1 h-screen w-full p-4">
                <WorldStructureEditor
                    name={name}
                    worldStructure={worldStructure}
                    setWorldStructure={setWorldStructure}
                />
            </div>
        </main>
    );
}