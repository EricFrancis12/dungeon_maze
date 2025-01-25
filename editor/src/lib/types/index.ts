export * from "./new";

export enum CellWall {
    None = "None",
    Solid = "Solid",
    SolidWithDoorGap = "SolidWithDoorGap",
    SolidWithWindowGap = "SolidWithWindowGap",
}

export enum CellSpecial {
    None = "None",
    Chair = "Chair",
    TreasureChest = "TreasureChest",
    Staircase = "Staircase",
    Stairs = "Stairs",
}

export type Cell = {
    wall_top: CellWall;
    wall_bottom: CellWall;
    wall_left: CellWall;
    wall_right: CellWall;
    floor: CellWall;
    ceiling: CellWall;
    door_top: boolean;
    door_bottom: boolean;
    door_left: boolean;
    door_right: boolean;
    window_top: boolean;
    window_bottom: boolean;
    window_left: boolean;
    window_right: boolean;
    special: CellSpecial;
};

export type Chunk = {
    x: number;
    y: number;
    z: number;
    cells: Cell[][];
    world_structure: string;
};

export type WorldStructure = {
    chunks: Chunk[];
};
