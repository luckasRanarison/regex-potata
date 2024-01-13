import { useEffect, useRef } from "react";

const NfaVisualizer = ({ svg }: { svg?: SVGSVGElement }) => {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (svg) {
      containerRef.current?.replaceChildren(svg);
    }
  }, [svg]);

  return (
    <div
      ref={containerRef}
      className="pt-12 pb-8 w-full overflow-scroll
      rounded-md border-[1px] border-slate-800"
    ></div>
  );
};

export default NfaVisualizer;
