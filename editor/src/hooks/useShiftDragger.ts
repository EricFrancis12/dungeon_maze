import { useEffect, useState } from "react";

export default function useShiftDragger(ref: React.MutableRefObject<HTMLElement | null>) {
    const [dragging, setDragging] = useState(false);
    const [position, setPosition] = useState<{ top: number; left: number }>({
        top: 0,
        left: 0,
    });

    useEffect(() => {
        function handleKeyDown(e: KeyboardEvent) {
            if (e.shiftKey) {
                setDragging(true);
            }
        }

        function handleKeyUp(e: KeyboardEvent) {
            if (!e.shiftKey) {
                setDragging(false);
            }
        }

        document.addEventListener("keydown", handleKeyDown);
        document.addEventListener("keyup", handleKeyUp);

        return () => {
            document.removeEventListener("keydown", handleKeyDown);
            document.removeEventListener("keyup", handleKeyUp);
        };
    }, []);

    useEffect(() => {
        let startX = 0;
        let startY = 0;
        let initialTop = 0;
        let initialLeft = 0;

        function onMouseDown(e: MouseEvent) {
            if (!dragging) return;

            if (ref.current) {
                startX = e.clientX;
                startY = e.clientY;
                initialTop = position.top;
                initialLeft = position.left;
                document.addEventListener("mousemove", onMouseMove);
                document.addEventListener("mouseup", onMouseUp);
            }
        }

        function onMouseMove(e: MouseEvent) {
            const deltaX = e.clientX - startX;
            const deltaY = e.clientY - startY;

            setPosition({
                top: initialTop + deltaY,
                left: initialLeft + deltaX,
            });
        }

        function onMouseUp() {
            document.removeEventListener("mousemove", onMouseMove);
            document.removeEventListener("mouseup", onMouseUp);
        }

        if (ref.current) {
            ref.current.addEventListener("mousedown", onMouseDown);
        }

        return () => {
            if (ref.current) {
                ref.current.removeEventListener("mousedown", onMouseDown);
            }
        };
    }, [dragging, position, ref]);

    return { position, setPosition, dragging };
}
