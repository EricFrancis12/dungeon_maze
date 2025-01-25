import { Button } from "../ui/Button";

export default function DummyChunkSection({ x, y, z, onClick }: {
    x: number;
    y: number;
    z: number;
    onClick: () => void;
}) {
    return (
        <div className="flex justify-center items-center h-full w-full bg-slate-300">
            <Button onClick={onClick}>
                {`add chunk: (${x}, ${y}, ${z})`}
            </Button>
        </div>
    );
}
