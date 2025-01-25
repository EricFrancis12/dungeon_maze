import CellSection from "./CellSection";
import { Chunk } from "../../lib/types";

export default function ChunkSection({ chunk, onChange }: {
    chunk: Chunk;
    onChange: (ch: Chunk) => void;
}) {
    return (
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
    );
}
