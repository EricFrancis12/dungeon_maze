import { z } from "zod";
import { Cell, CellSpecial, CellWall, Chunk, WorldStructure } from "./types";

export const cellSchema: z.ZodType<Cell> = z.object({
    wall_top: z.nativeEnum(CellWall),
    wall_bottom: z.nativeEnum(CellWall),
    wall_left: z.nativeEnum(CellWall),
    wall_right: z.nativeEnum(CellWall),
    floor: z.nativeEnum(CellWall),
    ceiling: z.nativeEnum(CellWall),
    door_top: z.boolean(),
    door_bottom: z.boolean(),
    door_left: z.boolean(),
    door_right: z.boolean(),
    window_top: z.boolean(),
    window_bottom: z.boolean(),
    window_left: z.boolean(),
    window_right: z.boolean(),
    special: z.nativeEnum(CellSpecial),
});

export const ChunkSchema: z.ZodType<Chunk> = z.object({
    x: z.number(),
    y: z.number(),
    z: z.number(),
    cells: z.array(z.array(cellSchema)),
    world_structure: z.string(),
});

export const WorldStructureSchema: z.ZodType<WorldStructure> = z.object({
    chunks: z.array(ChunkSchema),
});

