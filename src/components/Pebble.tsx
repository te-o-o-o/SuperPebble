import { memo, useMemo } from "react";
import rough from "roughjs";
import type { Options } from "roughjs/bin/core";
import { Handle, Position, type Edge, type EdgeProps, type Node, type NodeProps } from "@xyflow/react";
import type { PebbleData, RoughEdgeData } from "../layout";
import { COLORS, seed } from "../theme";
import { ScopeBadge } from "./ScopeBadge";

const gen = rough.generator();
const BG = [0x1a, 0x19, 0x18];

/** Opaque tint of `hex` over the canvas background, so edges don't show through pebbles. */
function tint(hex: string, alpha: number) {
  const c = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return `rgb(${c.map((v, i) => Math.round(v * alpha + BG[i] * (1 - alpha))).join(",")})`;
}

function pill(x: number, y: number, w: number, h: number) {
  const r = h / 2;
  return `M${x + r},${y} H${x + w - r} A${r},${r} 0 0 1 ${x + w - r},${y + h} H${x + r} A${r},${r} 0 0 1 ${x + r},${y} Z`;
}

/** `drawable` normalises the length to 1 so CSS can animate the stroke being drawn. */
function Paths({ paths, dashed, drawable }: { paths: ReturnType<typeof gen.toPaths>; dashed?: boolean; drawable?: boolean }) {
  return paths.map((p, i) => (
    <path key={i} pathLength={drawable ? 1 : undefined} d={p.d} stroke={p.stroke} strokeWidth={p.strokeWidth} fill={p.fill ?? "none"} strokeDasharray={dashed && p.stroke !== "none" ? "6 5" : undefined} />
  ));
}

function outline(variant: PebbleData["variant"], w: number, h: number, pad: number, opts: Options) {
  return variant === "item" || variant === "more"
    ? gen.path(pill(pad, pad, w - 2 * pad, h - 2 * pad), opts)
    : gen.ellipse(w / 2, h / 2, w - 2 * pad, h - 2 * pad, opts);
}

export const PebbleNode = memo(function PebbleNode({ id, data }: NodeProps<Node<PebbleData>>) {
  const { variant, w, h, color, dashed, selected } = data;
  const paths = useMemo(() => {
    const fill = variant === "more" ? undefined : tint(color, variant === "item" ? 0.1 : variant === "group" ? 0.2 : 0.25);
    const opts = { seed: seed(id), roughness: variant === "item" || variant === "more" ? 1.1 : 1.7, bowing: 1.4, stroke: color, strokeWidth: variant === "center" ? 2 : 1.5, fill, fillStyle: "solid" };
    return gen.toPaths(outline(variant, w, h, 4, opts));
  }, [id, variant, w, h, color]);
  // Pencil circle around the selection, a bit looser than the pebble itself.
  const halo = useMemo(() => {
    if (!selected) return null;
    const opts = { seed: seed(id) + 1, roughness: 2, bowing: 2, stroke: COLORS.text, strokeWidth: 1.2 };
    return gen.toPaths(outline(variant, w, h, variant === "item" ? -5 : -8, opts));
  }, [id, variant, w, h, selected]);

  const cls = ["pebble", variant, selected && "selected", data.dim && "dim", data.disabled && "disabled"].filter(Boolean).join(" ");
  return (
    <div className={cls} style={{ width: w, height: h }}>
      <svg width={w} height={h}>
        <Paths paths={paths} dashed={dashed} />
        {halo && (
          <g className="halo">
            <Paths paths={halo} drawable />
          </g>
        )}
      </svg>
      <div className="pebble-content">
        {data.scope && <ScopeBadge scope={data.scope} />}
        <span className="label">{data.label}</span>
        {data.sub && <span className="sub">{data.sub}</span>}
        {data.chip && (
          <span className="chip" style={{ background: data.chipColor }}>
            {data.chip}
          </span>
        )}
      </div>
      <Handle type="target" position={Position.Top} className="handle" />
      <Handle type="source" position={Position.Bottom} className="handle" />
    </div>
  );
});

export const RoughEdge = memo(function RoughEdge({ id, sourceX: sx, sourceY: sy, targetX: tx, targetY: ty, data }: EdgeProps<Edge<RoughEdgeData>>) {
  const color = data?.color ?? "#888";
  const paths = useMemo(() => {
    // Slight bend so branches read as drawn, not ruled.
    const mid: [number, number] = [(sx + tx) / 2 + (ty - sy) * 0.08, (sy + ty) / 2 - (tx - sx) * 0.08];
    return gen.toPaths(gen.curve([[sx, sy], mid, [tx, ty]], { seed: seed(id), roughness: 0.7, stroke: color, strokeWidth: 1.2 }));
  }, [id, sx, sy, tx, ty, color]);
  return (
    <g className="edge" style={{ opacity: data?.dim ? 0.12 : data?.active || data?.dashed ? 0.85 : 0.5 }}>
      <Paths paths={paths} dashed={data?.dashed} />
    </g>
  );
});
