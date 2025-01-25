import { Check } from "lucide-react";
import { ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger } from "@radix-ui/react-context-menu";
import {
    ContextMenu,
    ContextMenuContent,
    ContextMenuItem,
    ContextMenuTrigger,
} from "../../components/ui/ContextMenu";
import { cn } from "../../lib/utils";
import { Cell, CellSpecial, CellWall } from "../../lib/types";

export default function CellSection({ cell, onChange }: {
    cell: Cell;
    onChange: (c: Cell) => void;
}) {
    let bg = "white";
    if (cell.ceiling === CellWall.Solid) {
        if (cell.floor === CellWall.Solid) {
            bg = `repeating-linear-gradient(
                45deg,
                blue,
                blue 10px,
                red 10px,
                red 20px
            )`;
        } else {
            bg = "blue";
        }
    } else if (cell.floor === CellWall.Solid) {
        bg = "red";
    }

    return (
        <div
            className="relative border border-gray-500"
            style={{
                background: bg,
            }}
        >
            {
                cell.special === CellSpecial.Chair &&
                <div className="absolute flex justify-center items-center top-[37.5%] left-[37.5%] h-[25%] w-[25%] bg-white">
                    Ch
                </div>
            }
            {
                cell.special === CellSpecial.TreasureChest &&
                <div className="absolute flex justify-center items-center top-[37.5%] left-[30%] h-[25%] w-[40%] bg-amber-700">
                    TC
                </div>
            }
            {cell.special === CellSpecial.Staircase &&
                <div className="absolute grid grid-cols-6 top-[15%] left-[15%] h-[70%] w-[70%] border border-black">
                    {Array.from({ length: 6 }).map((_, index) => (
                        <div key={index} className={(index === 0 ? "" : "border-l") + " h-full border-black"}>

                        </div>
                    ))}
                </div>
            }
            {cell.special === CellSpecial.Stairs &&
                <div className="absolute flex flex-col justify-between top-[15%] left-[15%] h-[70%] w-[70%] border border-black">
                    {Array.from({ length: 2 }).map((_, index) => (
                        <div key={index} className="grid grid-cols-8 h-[40%] w-full border border-black">
                            {Array.from({ length: 8 }).map((__, _index) => (
                                <div key={index} className={(_index === 0 ? "" : "border-l") + " h-full border-black"}>

                                </div>
                            ))}
                        </div>
                    ))}
                </div>
            }
            <div className="absolute top-[10%] left-[10%] h-[80%] w-[80%] ">
                <ContextMenu>
                    <ContextMenuTrigger>
                        <div className="relative h-full w-full">
                            <div className="absolute top-0 left-0 h-full w-full cursor-pointer bg-slate-300 opacity-0 hover:opacity-70" />
                        </div>
                    </ContextMenuTrigger>
                    <ContextMenuContent className="bg-white">
                        <ContextMenuItem
                            onClick={() => onChange({
                                ...cell,
                                ceiling: cell.ceiling === CellWall.Solid ? CellWall.None : CellWall.Solid,
                            })}
                        >
                            {cell.ceiling === CellWall.Solid && <Check />}
                            Ceiling
                        </ContextMenuItem>
                        <ContextMenuItem
                            onClick={() => onChange({
                                ...cell,
                                floor: cell.floor === CellWall.Solid ? CellWall.None : CellWall.Solid,
                            })}
                        >
                            {cell.floor === CellWall.Solid && <Check />}
                            Floor
                        </ContextMenuItem>
                        <ContextMenuSub>
                            <ContextMenuSubTrigger>Special</ContextMenuSubTrigger>
                            <ContextMenuSubContent className="bg-white">
                                {Object.values(CellSpecial).map((cellSpecial) => (
                                    <ContextMenuItem
                                        key={cellSpecial}
                                        onClick={() => onChange({
                                            ...cell,
                                            special: cellSpecial,
                                        })}
                                    >
                                        {cell.special === cellSpecial && <Check />}
                                        {cellSpecial}
                                    </ContextMenuItem>
                                ))}
                            </ContextMenuSubContent>
                        </ContextMenuSub>
                    </ContextMenuContent>
                </ContextMenu>
            </div>
            <WallSection
                cell={cell}
                side="Top"
                onChange={onChange}
            />
            <WallSection
                cell={cell}
                side="Bottom"
                onChange={onChange}
            />
            <WallSection
                cell={cell}
                side="Left"
                onChange={onChange}
            />
            <WallSection
                cell={cell}
                side="Right"
                onChange={onChange}
            />
        </div>
    );
}

type Side = "Top" | "Bottom" | "Left" | "Right";

function getCellWallProp(side: Side): keyof Cell {
    switch (side) {
        case "Top": return "wall_top";
        case "Bottom": return "wall_bottom";
        case "Left": return "wall_left";
        case "Right": return "wall_right";
    }
}

function getCellDoorProp(side: Side): keyof Cell {
    switch (side) {
        case "Top": return "door_top";
        case "Bottom": return "door_bottom";
        case "Left": return "door_left";
        case "Right": return "door_right";
    }
}

function getCellWindowProp(side: Side): keyof Cell {
    switch (side) {
        case "Top": return "window_top";
        case "Bottom": return "window_bottom";
        case "Left": return "window_left";
        case "Right": return "window_right";
    }
}

function WallSection({ cell, side, onChange }: {
    cell: Cell;
    side: Side;
    onChange: (c: Cell) => void;
}) {
    const wallProp = getCellWallProp(side);
    const doorProp = getCellDoorProp(side);
    const windowProp = getCellWindowProp(side);

    return (
        <WallSectionWrapper side={side}>
            <ContextMenu>
                <ContextMenuTrigger>
                    <div className="relative h-full w-full">
                        <div className="absolute top-0 left-0 h-full w-full cursor-pointer bg-slate-300 opacity-0 hover:opacity-70" />
                        {cell[wallProp] === CellWall.Solid && <WallSolidDecoration side={side} />}
                        {cell[wallProp] === CellWall.SolidWithDoorGap && <WallSolidWithDoorGapDecoration side={side} />}
                        {cell[wallProp] === CellWall.SolidWithWindowGap && <WallSolidWithWindowGapDecoration side={side} />}
                        {cell[windowProp] && <WindowDecoration side={side} />}
                        {cell[doorProp] && <DoorDecoration side={side} />}
                    </div>
                </ContextMenuTrigger>
                <ContextMenuContent className="bg-white">
                    <ContextMenuSub>
                        <ContextMenuSubTrigger>{`Wall ${side}`}</ContextMenuSubTrigger>
                        <ContextMenuSubContent className="bg-white">
                            {Object.values(CellWall).map((cellWall) => (
                                <ContextMenuItem
                                    key={cellWall}
                                    onClick={() => onChange({
                                        ...cell,
                                        [wallProp]: cellWall,
                                    })}
                                >
                                    {cell[wallProp] === cellWall && <Check />}
                                    {cellWall}
                                </ContextMenuItem>
                            ))}
                        </ContextMenuSubContent>
                    </ContextMenuSub>
                    <ContextMenuItem
                        onClick={() => onChange({
                            ...cell,
                            [doorProp]: !cell[doorProp],
                        })}
                    >
                        {cell[doorProp] && <Check />}
                        {`Door ${side}`}
                    </ContextMenuItem>
                    <ContextMenuItem
                        onClick={() => onChange({
                            ...cell,
                            [windowProp]: !cell[windowProp],
                        })}
                    >
                        {cell[windowProp] && <Check />}
                        {`Window ${side}`}
                    </ContextMenuItem>
                </ContextMenuContent>
            </ContextMenu>
        </WallSectionWrapper>
    );
}

function WallSectionWrapper({ side, children }: {
    side: Side;
    children: React.ReactNode;
}) {
    return (
        <div
            className={cn(
                "absolute",
                side === "Top" ? "top-0 left-0 h-[10%] w-full" : "",
                side === "Bottom" ? "bottom-0 left-0 h-[10%] w-full" : "",
                side === "Left" ? "top-0 left-0 h-full w-[10%]" : "",
                side === "Right" ? "top-0 right-0 h-full w-[10%]" : "",
            )}
        >
            {children}
        </div>
    );
}

function WallSolidDecoration({ side }: {
    side: Side;
}) {
    return (
        <div
            className={cn(
                "absolute bg-black z-40",
                side === "Top" ? "top-0 left-0 h-[25%] w-full" : "",
                side === "Bottom" ? "bottom-0 left-0 h-[25%] w-full" : "",
                side === "Left" ? "top-0 left-0 h-full w-[25%]" : "",
                side === "Right" ? "top-0 right-0 h-full w-[25%]" : "",
            )}
        />
    );
}

function WallSolidWithDoorGapDecoration({ side }: {
    side: Side;
}) {
    return (
        <>
            <div
                className={cn(
                    "absolute bg-black z-30",
                    side === "Top" ? "top-0 left-0 h-[25%] w-[37.5%]" : "",
                    side === "Bottom" ? "bottom-0 left-0 h-[25%] w-[37.5%]" : "",
                    side === "Left" ? "top-0 left-0 h-[37.5%] w-[25%]" : "",
                    side === "Right" ? "top-0 right-0 h-[37.5%] w-[25%]" : "",
                )}
            />
            <div
                className={cn(
                    "absolute bg-black z-30",
                    side === "Top" ? "top-0 right-0 h-[25%] w-[37.5%]" : "",
                    side === "Bottom" ? "bottom-0 right-0 h-[25%] w-[37.5%]" : "",
                    side === "Left" ? "bottom-0 left-0 h-[37.5%] w-[25%]" : "",
                    side === "Right" ? "bottom-0 right-0 h-[37.5%] w-[25%]" : "",
                )}
            />
        </>
    );
}

function WallSolidWithWindowGapDecoration({ side }: {
    side: Side;
}) {
    return (
        <>
            <div
                className={cn(
                    "absolute bg-black z-30",
                    side === "Top" ? "top-0 left-0 h-[25%] w-[40%]" : "",
                    side === "Bottom" ? "bottom-0 left-0 h-[25%] w-[40%]" : "",
                    side === "Left" ? "top-0 left-0 h-[40%] w-[25%]" : "",
                    side === "Right" ? "top-0 right-0 h-[40%] w-[25%]" : "",
                )}
            />
            <div
                className={cn(
                    "absolute bg-black z-30",
                    side === "Top" ? "top-0 right-0 h-[25%] w-[40%]" : "",
                    side === "Bottom" ? "bottom-0 right-0 h-[25%] w-[40%]" : "",
                    side === "Left" ? "bottom-0 left-0 h-[40%] w-[25%]" : "",
                    side === "Right" ? "bottom-0 right-0 h-[40%] w-[25%]" : "",
                )}
            />
        </>
    );
}

function WindowDecoration({ side }: {
    side: Side;
}) {
    return (
        <div
            className={cn(
                "absolute bg-yellow-300 z-20",
                side === "Top" ? "top-0 left-[40%] h-[50%] w-[20%]" : "",
                side === "Bottom" ? "bottom-0 left-[40%] h-[50%] w-[20%]" : "",
                side === "Left" ? "top-[40%] left-0 h-[20%] w-[50%]" : "",
                side === "Right" ? "top-[40%] right-0 h-[20%] w-[50%]" : "",
            )}
        />
    );
}

function DoorDecoration({ side }: {
    side: Side;
}) {
    return (
        <div
            className={cn(
                "absolute bg-amber-900 z-10",
                side === "Top" ? "top-0 left-[37.5%] h-full w-[25%]" : "",
                side === "Bottom" ? "bottom-0 left-[37.5%] h-full w-[25%]" : "",
                side === "Left" ? "top-[37.5%] left-0 h-[25%] w-full" : "",
                side === "Right" ? "top-[37.5%] right-0 h-[25%] w-full" : "",
            )}
        />
    );
}
