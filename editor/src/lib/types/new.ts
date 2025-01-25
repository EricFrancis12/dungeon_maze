import { Cell, CellSpecial, CellWall, Chunk, WorldStructure } from ".";

export function newWorldStructure(): WorldStructure {
    return {
        chunks: [],
    };
}

export function newChunk(x: number, y: number, z: number): Chunk {
    return {
        x,
        y,
        z,
        cells: new Array(4).fill(new Array(4).fill(newCell())),
        world_structure: "None",
    };
}

export function newCell(): Cell {
    return {
        wall_top: CellWall.None,
        wall_bottom: CellWall.None,
        wall_left: CellWall.None,
        wall_right: CellWall.None,
        floor: CellWall.None,
        ceiling: CellWall.None,
        door_top: false,
        door_bottom: false,
        door_left: false,
        door_right: false,
        window_top: false,
        window_bottom: false,
        window_left: false,
        window_right: false,
        special: CellSpecial.None,
    };
}
