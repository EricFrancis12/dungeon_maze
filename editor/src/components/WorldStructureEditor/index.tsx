import { Fragment, useRef, useState } from "react";
import useScrollScaler from "../../hooks/useScrollScaler";
import useShiftDragger from "../../hooks/useShiftDragger";
import ChunkSection from "./ChunkSection";
import { Chunk, WorldStructure, newChunk } from "../../lib/types";

const MIN_RADIUS = 2;

export default function WorldStructureEditor({ worldStructure, setWorldStructure }: {
    worldStructure: WorldStructure;
    setWorldStructure: (ws: WorldStructure) => void;
}) {
    const ref = useRef<HTMLDivElement>(null);
    const { position, setPosition, dragging } = useShiftDragger(ref);

    const boundsRef = useRef<HTMLDivElement>(null);
    useScrollScaler(ref, boundsRef);

    const [chunkY, setChunkY] = useState(0);

    function handleChange(e: React.ChangeEvent<HTMLSelectElement>) {
        const newChunkY = parseInt(e.target.value);
        if (!isNaN(newChunkY)) {
            setChunkY(newChunkY);
        }
    }

    let radius = worldStructure.chunks.reduce((acc: number, curr) => {
        let max = acc;
        if (Math.abs(curr.x) > acc) max = curr.x;
        if (Math.abs(curr.y) > acc) max = curr.y;
        if (Math.abs(curr.z) > acc) max = curr.z;
        return max;
    }, 0);

    if (radius < MIN_RADIUS) radius = MIN_RADIUS;

    const chunks: (Chunk | { x: number; z: number; })[][] = [];
    for (let z = -radius; z <= radius; z++) {
        const row: (Chunk | { x: number; z: number; })[] = [];
        for (let x = -radius; x <= radius; x++) {
            const chunk = worldStructure.chunks.find((chunk => chunk.x === x && chunk.y === chunkY && chunk.z === z));
            row.push(chunk ?? { x, z });
        }
        chunks.push(row);
    }

    return (
        <div className="flex flex-col h-full w-full">
            <div className="flex items-center gap-2 w-full p-2">
                <button onClick={() => setPosition({ top: 0, left: 0 })}>{"To (0,0)"}</button>
                <select value={chunkY} onChange={handleChange}>
                    {[-2, -1, 0, 1, 2].map((y) => (
                        <option key={y} value={y}>{y}</option>
                    ))}
                </select>
            </div>
            <div
                ref={boundsRef}
                className={(dragging ? "cursor-pointer" : "cursor-default")
                    + " relative flex-1 h-full w-full bg-purple-400 overflow-hidden"}
            >
                <div
                    ref={ref}
                    className="absolute grid grid-cols-5 h-[2000px] w-[2000px] bg-white"
                    style={{
                        top: position.top,
                        left: position.left,
                    }}
                >
                    {chunks.map((row) => row.map((chunk) => (
                        <Fragment key={`${chunk.x} ${chunk.z}`}>
                            <div className="relative flex justify-center items-center border border-gray-500">
                                {"cells" in chunk
                                    ? <ChunkSection
                                        chunk={chunk}
                                        onChange={(newChunk) => setWorldStructure({
                                            ...worldStructure,
                                            chunks: worldStructure.chunks.map(
                                                (ch) => (ch.x === newChunk.x && ch.y === newChunk.y && ch.z === newChunk.z)
                                                    ? newChunk
                                                    : ch
                                            ),
                                        })}
                                    />
                                    : <button
                                        onClick={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: [
                                                ...worldStructure.chunks,
                                                newChunk(chunk.x, chunkY, chunk.z),
                                            ],
                                        })}
                                    >
                                        {`add chunk: (${chunk.x}, ${chunkY}, ${chunk.z})`}
                                    </button>
                                }
                                {[[radius, "top"], [-radius, "bottom"]].map(([r, d], index) => chunk.z === r
                                    ? (
                                        <div
                                            key={index}
                                            className="absolute flex justify-center items-center left-0 h-[16%] w-full bg-purple-200"
                                            style={{ [d]: "100%" }}
                                        >
                                            {`X: ${chunk.x}`}
                                        </div>
                                    )
                                    : null
                                )}
                                {[[radius, "left"], [-radius, "right"]].map(([r, d], index) => chunk.x === r
                                    ? (
                                        <div
                                            key={index}
                                            className="absolute flex justify-center items-center bottom-0 h-full w-[16%] bg-purple-200"
                                            style={{ [d]: "100%" }}
                                        >
                                            {`Z: ${chunk.z}`}
                                        </div>
                                    )
                                    : null
                                )}
                            </div>
                        </Fragment>
                    )))}
                </div>
                {dragging && <div className="h-full w-full bg-white opacity-30 pointer-events-none" />}
            </div>
        </div>
    );
}
