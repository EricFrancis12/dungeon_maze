import {
    ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
    ContextMenuSub, ContextMenuSubTrigger, ContextMenuSubContent,
} from "@radix-ui/react-context-menu";
import CellSection from "./CellSection";
import { CellSpecial, CellWall, Chunk } from "../../lib/types";
import { cn } from "../../lib/utils";

export default function ChunkSection({ chunk, onChange, onDeleteIntent, onOriginChunkChangeIntent }: {
    chunk: Chunk;
    onChange: (ch: Chunk) => void;
    onDeleteIntent?: () => void;
    onOriginChunkChangeIntent?: () => void;
}) {
    return (
        <div className="relative h-full w-full">
            <div className="grid grid-cols-4 h-full w-full">
                {chunk.cells.map((row, cellZ) => row.map((cell, cellX) => (
                    <CellSection
                        key={`${cellX} ${cellZ}`}
                        cell={cell}
                        onChange={(newCell) => onChange({
                            ...chunk,
                            cells: chunk.cells.map(
                                (rw, cz) => cz === cellZ
                                    ? rw.map(
                                        (c, cx) => cx === cellX
                                            ? newCell
                                            : c
                                    )
                                    : rw
                            )
                        })}
                    />
                )))}
            </div>
            <ContextMenu>
                <ContextMenuTrigger>
                    <div
                        className={cn(
                            "absolute top-0 right-0 h-[6%] w-[6%] rounded-bl-full cursor-pointer z-50 opacity-80 hover:opacity-100",
                            chunk.world_structure === "None" ? "bg-blue-400" : "bg-green-400"
                        )}
                    />
                </ContextMenuTrigger>
                <ContextMenuContent className="bg-white z-60">
                    <ContextMenuItem onClick={onOriginChunkChangeIntent}>
                        Set As Origin Chunk
                    </ContextMenuItem>
                    <ContextMenuItem onClick={onDeleteIntent}>
                        Delete Chunk
                    </ContextMenuItem>
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>Fill Walls</ContextMenuSubTrigger>
                        <ContextMenuSubContent className="bg-white">
                            {Object.values(CellWall).map((cellWall) => (
                                <ContextMenuItem
                                    key={cellWall}
                                    onClick={() => onChange({
                                        ...chunk,
                                        cells: chunk.cells.map((row) => row.map((c) => ({
                                            ...c,
                                            wall_top: cellWall,
                                            wall_bottom: cellWall,
                                            wall_left: cellWall,
                                            wall_right: cellWall,
                                        }))),
                                    })}
                                >
                                    {cellWall}
                                </ContextMenuItem>
                            ))}
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>Fill Special</ContextMenuSubTrigger>
                        <ContextMenuSubContent className="bg-white">
                            {Object.values(CellSpecial).map((cellSpecial) => (
                                <ContextMenuItem
                                    key={cellSpecial}
                                    onClick={() => onChange({
                                        ...chunk,
                                        cells: chunk.cells.map((row) => row.map((c) => ({
                                            ...c,
                                            special: cellSpecial,
                                        }))),
                                    })}
                                >
                                    {cellSpecial}
                                </ContextMenuItem>
                            ))}
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>Fill Ceilings</ContextMenuSubTrigger>
                        <ContextMenuSubContent className="bg-white">
                            {[CellWall.None, CellWall.Solid].map((cellWall) => (
                                <ContextMenuItem
                                    key={cellWall}
                                    onClick={() => onChange({
                                        ...chunk,
                                        cells: chunk.cells.map((row) => row.map((c) => ({
                                            ...c,
                                            ceiling: cellWall,
                                        }))),
                                    })}
                                >
                                    {cellWall}
                                </ContextMenuItem>
                            ))}
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>Fill Floors</ContextMenuSubTrigger>
                        <ContextMenuSubContent className="bg-white">
                            {[CellWall.None, CellWall.Solid].map((cellWall) => (
                                <ContextMenuItem
                                    key={cellWall}
                                    onClick={() => onChange({
                                        ...chunk,
                                        cells: chunk.cells.map((row) => row.map((c) => ({
                                            ...c,
                                            floor: cellWall,
                                        }))),
                                    })}
                                >
                                    {cellWall}
                                </ContextMenuItem>
                            ))}
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                </ContextMenuContent>
            </ContextMenu>
        </div>
    );
}
