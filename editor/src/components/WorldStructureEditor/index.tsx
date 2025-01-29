import { CSSProperties, Fragment, useRef, useState } from "react";
import { Plus } from "lucide-react";
import useScrollScaler from "../../hooks/useScrollScaler";
import useShiftDragger from "../../hooks/useShiftDragger";
import { Button } from "../ui/Button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/Select"
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger } from "../../components/ui/ContextMenu";
import ChunkSection from "./ChunkSection";
import DummyChunkSection from "./DummyChunkSection";
import { arrFromIncr, cn } from "../../lib/utils";
import { Cell, CellWall, Chunk, WorldStructure, newChunk } from "../../lib/types";

type DummyChunk = {
    x: number;
    z: number;
};

function calcXZRadius(ws: WorldStructure): number {
    return ws.chunks.reduce((acc: number, curr) => {
        let max = acc;

        const absX = Math.abs(curr.x);
        if (absX > acc) max = absX;

        const absZ = Math.abs(curr.z);
        if (absZ > acc) max = absZ;

        return max;
    }, 0);
}

export default function WorldStructureEditor({ name, worldStructure, setWorldStructure }: {
    name: string;
    worldStructure: WorldStructure;
    setWorldStructure: (ws: WorldStructure) => void;
}) {
    const ref = useRef<HTMLDivElement>(null);
    const { position, setPosition, dragging } = useShiftDragger(ref);

    const boundsRef = useRef<HTMLDivElement>(null);
    useScrollScaler(ref, boundsRef);

    const [chunkY, setChunkY] = useState(0);

    function handleChange(value: string) {
        const newChunkY = parseInt(value);
        if (!isNaN(newChunkY)) {
            setChunkY(newChunkY);
        }
    }

    const xzRadius = calcXZRadius(worldStructure);

    const chunks: (Chunk | DummyChunk)[][] = [];
    for (let z = -xzRadius; z <= xzRadius; z++) {
        const row: (Chunk | DummyChunk)[] = [];
        for (let x = -xzRadius; x <= xzRadius; x++) {
            const chunk = worldStructure.chunks.find((chunk => chunk.x === x && chunk.y === chunkY && chunk.z === z));
            row.push(chunk ?? { x, z });
        }
        chunks.push(row);
    }

    let leastY = 0;
    let greatestY = 0;
    for (const chunk of worldStructure.chunks) {
        if (chunk.y < leastY) {
            leastY = chunk.y;
        } else if (chunk.y > greatestY) {
            greatestY = chunk.y;
        }
    }

    const yChoices = arrFromIncr(leastY, greatestY, 1);

    function handleChunkUpdate(
        chunkPredicate: (ch: Chunk) => boolean,
        newCellFunc: (c: Cell) => Cell,
    ): void {
        setWorldStructure({
            ...worldStructure,
            chunks: worldStructure.chunks.map(
                (ch) => chunkPredicate(ch)
                    ? { ...ch, cells: ch.cells.map((row) => row.map(newCellFunc)) }
                    : ch,
            ),
        })
    };

    return (
        <div className="flex flex-col h-full w-full">
            <div className="flex flex-col gap-2 w-full p-2">
                <div className="flex items-center gap-4">
                    <div className="flex items-center gap-2">
                        <span className="whitespace-nowrap">Chunk Y:</span>
                        <Select
                            value={`${chunkY}`}
                            onValueChange={handleChange}
                        >
                            <SelectTrigger className="max-w-[100px]">
                                <SelectValue>
                                    {chunkY}
                                </SelectValue>
                            </SelectTrigger>
                            <SelectContent className="bg-white">
                                {yChoices.map((y) => (
                                    <SelectItem key={y} value={`${y}`}>{y}</SelectItem>
                                ))}
                            </SelectContent>
                        </Select>
                    </div>
                    <Button onClick={() => setPosition({ top: 0, left: 0 })}>Reset View</Button>
                </div>
                <div className="flex items-center gap-4">
                    <Button
                        onClick={() => setWorldStructure({
                            ...worldStructure,
                            chunks: [
                                newChunk(0, leastY - 1, 0),
                                ...worldStructure.chunks,
                            ],
                        })}
                    >
                        {`Extend -Y to ${leastY - 1} (create a new chunk at (0, ${leastY - 1}, 0))`}
                    </Button>
                    <Button
                        onClick={() => setWorldStructure({
                            ...worldStructure,
                            chunks: [
                                ...worldStructure.chunks,
                                newChunk(0, greatestY + 1, 0),
                            ],
                        })}
                    >
                        {`Extend +Y to ${greatestY + 1} (create a new chunk at (0, ${greatestY + 1}, 0))`}
                    </Button>
                </div>
            </div>
            <div
                ref={boundsRef}
                className={(dragging ? "cursor-pointer" : "cursor-default")
                    + " relative flex-1 h-full w-full bg-purple-400 overflow-hidden"}
            >
                <div
                    ref={ref}
                    className="absolute grid bg-white"
                    style={{
                        gridTemplateColumns: `repeat(${xzRadius * 2 + 1}, minmax(0, 1fr))`,
                        top: position.top,
                        left: position.left,
                        height: `${400 * (xzRadius + 1)}px`,
                        width: `${400 * (xzRadius + 1)}px`,
                    }}
                >
                    {chunks.map((row, rowIndex) => row.map((chunk, chunkIndex) => (
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
                                        onDeleteIntent={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: worldStructure.chunks.filter(
                                                (ch) => (ch.x !== chunk.x || ch.y !== chunk.y || ch.z !== chunk.z)
                                            ),
                                        })}
                                        onOriginChunkChangeIntent={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: worldStructure.chunks.map(
                                                (ch) => ({
                                                    ...ch,
                                                    world_structure: (ch.x === chunk.x && ch.y === chunk.y && ch.z === chunk.z)
                                                        ? name
                                                        : "None",
                                                })
                                            ),
                                        })}
                                    />
                                    : <DummyChunkSection
                                        x={chunk.x}
                                        y={chunkY}
                                        z={chunk.z}
                                        onClick={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: [
                                                ...worldStructure.chunks,
                                                newChunk(chunk.x, chunkY, chunk.z),
                                            ],
                                        })}
                                    />
                                }
                                {([[xzRadius, "top"], [-xzRadius, "bottom"]] as const).map(([r, d], index) => chunk.z === r
                                    ? <ColRowHeading
                                        key={index}
                                        text={`X: ${chunk.x}`}
                                        className="left-0 h-[16%] w-full"
                                        style={{ [d]: "100%" }}
                                        onClearIntent={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: worldStructure.chunks.filter(
                                                (ch) => ch.x !== chunk.x
                                            ),
                                        })}
                                        onAllCeilingsIntent={() => handleChunkUpdate(
                                            (ch) => ch.x === chunk.x,
                                            (c) => ({ ...c, ceiling: CellWall.Solid }),
                                        )}
                                        onClearCeilingsIntent={() => handleChunkUpdate(
                                            (ch) => ch.x === chunk.x,
                                            (c) => ({ ...c, ceiling: CellWall.None }),
                                        )}
                                        onAllFlooredIntent={() => handleChunkUpdate(
                                            (ch) => ch.x === chunk.x,
                                            (c) => ({ ...c, floor: CellWall.Solid }),
                                        )}
                                        onClearFlooredIntent={() => handleChunkUpdate(
                                            (ch) => ch.x === chunk.x,
                                            (c) => ({ ...c, floor: CellWall.None }),
                                        )}
                                    />
                                    : null
                                )}
                                {([[xzRadius, "left"], [-xzRadius, "right"]] as const).map(([r, d], index) => chunk.x === r
                                    ? <ColRowHeading
                                        key={index}
                                        text={`Z: ${chunk.z}`}
                                        className="bottom-0 h-full w-[16%]"
                                        style={{ [d]: "100%" }}
                                        onClearIntent={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: worldStructure.chunks.filter(
                                                (ch) => ch.z !== chunk.z
                                            ),
                                        })}
                                        onAllCeilingsIntent={() => handleChunkUpdate(
                                            (ch) => ch.z === chunk.z,
                                            (c) => ({ ...c, ceiling: CellWall.Solid }),
                                        )}
                                        onClearCeilingsIntent={() => handleChunkUpdate(
                                            (ch) => ch.z === chunk.z,
                                            (c) => ({ ...c, ceiling: CellWall.None }),
                                        )}
                                        onAllFlooredIntent={() => handleChunkUpdate(
                                            (ch) => ch.z === chunk.z,
                                            (c) => ({ ...c, floor: CellWall.Solid }),
                                        )}
                                        onClearFlooredIntent={() => handleChunkUpdate(
                                            (ch) => ch.z === chunk.z,
                                            (c) => ({ ...c, floor: CellWall.None }),
                                        )}
                                    />
                                    : null
                                )}

                                {/* Top row plus buttons */}
                                {rowIndex === 0 &&
                                    <PlusButton
                                        className="top-[-50%] left-[40%]"
                                        onClick={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: [
                                                newChunk(chunk.x, chunkY, chunk.z - 1),
                                                ...worldStructure.chunks,
                                            ],
                                        })}
                                    />
                                }

                                {/* Bottom row plus buttons */}
                                {(rowIndex === chunks.length - 1) &&
                                    <PlusButton
                                        className="bottom-[-50%] left-[40%]"
                                        onClick={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: [
                                                ...worldStructure.chunks,
                                                newChunk(chunk.x, chunkY, chunk.z + 1),
                                            ],
                                        })}
                                    />
                                }

                                {/* Left column plus buttons */}
                                {chunkIndex === 0 &&
                                    <PlusButton
                                        className="top-[40%] left-[-50%]"
                                        onClick={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: [
                                                newChunk(chunk.x - 1, chunkY, chunk.z),
                                                ...worldStructure.chunks,
                                            ],
                                        })}
                                    />
                                }

                                {/* Right column plus buttons */}
                                {(chunkIndex === row.length - 1) &&
                                    <PlusButton
                                        className="top-[40%] right-[-50%]"
                                        onClick={() => setWorldStructure({
                                            ...worldStructure,
                                            chunks: [
                                                ...worldStructure.chunks,
                                                newChunk(chunk.x + 1, chunkY, chunk.z),
                                            ],
                                        })}
                                    />
                                }
                            </div>
                        </Fragment>
                    )))}
                </div>
                {dragging && <div className="h-full w-full bg-white opacity-30 pointer-events-none" />}
            </div>
        </div >
    );
}

function ColRowHeading({ text, onClearIntent, onAllCeilingsIntent, onClearCeilingsIntent, onAllFlooredIntent, onClearFlooredIntent, className, style }: {
    text: string;
    onClearIntent: () => void;
    onAllCeilingsIntent: () => void;
    onClearCeilingsIntent: () => void;
    onAllFlooredIntent: () => void;
    onClearFlooredIntent: () => void;
    className?: string;
    style?: CSSProperties;
}) {
    return (
        <ContextMenu>
            <ContextMenuTrigger>
                <div
                    className={cn(
                        "absolute flex justify-center items-center bg-purple-200",
                        className,
                    )}
                    style={style}
                >
                    {text}
                </div>
            </ContextMenuTrigger>
            <ContextMenuContent className="bg-white">
                <ContextMenuItem onClick={onClearIntent}>
                    Clear
                </ContextMenuItem>
                <ContextMenuItem onClick={onAllCeilingsIntent}>
                    All Ceilings
                </ContextMenuItem>
                <ContextMenuItem onClick={onClearCeilingsIntent}>
                    Clear Ceilings
                </ContextMenuItem>
                <ContextMenuItem onClick={onAllFlooredIntent}>
                    All Floors
                </ContextMenuItem>
                <ContextMenuItem onClick={onClearFlooredIntent}>
                    Clear Floors
                </ContextMenuItem>
            </ContextMenuContent>
        </ContextMenu>
    );
}

function PlusButton({ onClick, className }: {
    onClick: () => void;
    className?: string;
}) {
    return (
        <div
            className={cn(
                "absolute flex justify-center items-center h-[20%] w-[20%] bg-slate-200 rounded-full cursor-pointer hover:opacity-70",
                className,
            )}
            onClick={onClick}
        >
            <Plus />
        </div>
    );
}
