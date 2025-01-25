import { useEffect, useState } from "react";

export default function useScrollScaler(
    scaleRef: React.MutableRefObject<HTMLElement | null>,
    boundsRef: React.MutableRefObject<HTMLElement | null>,
) {
    const [scale, setScale] = useState(1);
    const [active, setActive] = useState(false);

    useEffect(() => {
        function handleMouseEnter() {
            setActive(true);
        }

        function handleMouseLeave() {
            setActive(false);
        }

        boundsRef.current?.addEventListener("mouseenter", handleMouseEnter);
        boundsRef.current?.addEventListener("mouseleave", handleMouseLeave);

        return () => {
            boundsRef.current?.removeEventListener("mouseenter", handleMouseEnter);
            boundsRef.current?.removeEventListener("mouseleave", handleMouseLeave);
        };
    }, [boundsRef.current]);

    useEffect(() => {
        function handleWheel(e: WheelEvent) {
            if (active && scaleRef.current) {
                e?.preventDefault();
                const newScale = e.deltaY > 0 ? scale - 0.1 : e.deltaY < 0 ? scale + 0.1 : scale;
                setScale(newScale);
                scaleRef.current.style.scale = scale.toString();
            }
        }

        document.addEventListener("wheel", handleWheel, { passive: false });

        return () => document.removeEventListener("wheel", handleWheel);
    }, [scale, active]);

    return scale;
}
