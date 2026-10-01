import type { Edge, Node } from "@xyflow/react";
import { COLORS, GROUPS, RULE_CHIP, fmtTokens, groupOf, hint, issuesByNode, type GroupKey } from "./theme";
import { t } from "./i18n";
import type { Graph, PNode, Scope } from "./types";

export type PebbleData = {
  variant: "center" | "group" | "item" | "more";
  label: string;
  sub?: string;
  color: string;
  scope?: Scope;
  chip?: string;
  chipColor?: string;
  dashed?: boolean;
  disabled?: boolean;
  selected?: boolean;
  /** Something else is selected and this isn't linked to it. */
  dim?: boolean;
  /** Node id for items, group key for groups and "+N more" / "collapse". */
  target: string;
  w: number;
  h: number;
};

export type RoughEdgeData = { color: string; dashed?: boolean; active?: boolean; dim?: boolean };

const GROUP_RADIUS = 260;
const ITEM_DIST = 190;
/** Items shown per collapsed group, the rest behind "+N more". */
const COLLAPSED_MAX = 5;
const PER_COLUMN = 6;
/** Minimum space kept between two pebbles. */
const GAP = 8;

const rad = (deg: number) => (deg * Math.PI) / 180;

/**
 * Deterministic radial layout: groups on a ring around Claude Code, items stacked in columns
 * perpendicular to their branch. Neighbouring branches can still meet: see `fan`.
 */
function column(origin: { x: number; y: number }, angle: number, count: number, maxW = 180) {
  const a = rad(angle);
  const [dx, dy] = [Math.cos(a), Math.sin(a)];
  const step = 50 + (maxW - 30) * Math.abs(dy); // items side by side need their width, stacked ones their height
  const colStep = 220 * Math.abs(dx) + 56 * Math.abs(dy);
  // Side by side, six pills sprawl over a thousand pixels: stack more, shorter rows instead.
  const per = Math.abs(dy) > 0.7 ? 3 : PER_COLUMN;
  return Array.from({ length: count }, (_, i) => {
    const col = Math.floor(i / per);
    const inCol = Math.min(per, count - col * per);
    const t = (i % per - (inCol - 1) / 2) * step;
    const dist = ITEM_DIST + col * colStep - Math.abs(t) * 0.12;
    return { x: origin.x + dx * dist - dy * t, y: origin.y + dy * dist + dx * t };
  });
}

function itemSize(label: string, sub = "", chip = "") {
  const w = 58 + label.length * 7.4 + (sub ? sub.length * 6.4 + 14 : 0) + (chip ? chip.length * 6.4 + 22 : 0);
  return { w: Math.min(280, Math.max(120, w)), h: 40 };
}

/** `scope` keeps only that scope's items (plugin content follows its plugin). */
export function layout(graph: Graph, hidden: Set<GroupKey>, expanded: Set<string>, selected: string | null, scope: Scope | null) {
  const inScope = (n: PNode) => !scope || n.scope === scope;
  const nodes: Node<PebbleData>[] = [];
  const edges: Edge<RoughEdgeData>[] = [];
  const issues = issuesByNode(graph.issues);
  const children = new Map<string, PNode[]>();
  for (const n of graph.nodes) if (n.parent) children.set(n.parent, [...(children.get(n.parent) ?? []), n]);

  const total = graph.nodes.filter((n) => n.enabled && n.kind !== "plugin" && inScope(n)).reduce((s, n) => s + (n.tokens ?? 0), 0);
  nodes.push(pebble("center", { x: 0, y: 0 }, { variant: "center", label: "Claude Code", sub: `~${fmtTokens(total)} tok`, color: COLORS.center, target: "center", w: 124, h: 124 }));

  const itemData = (n: PNode): PebbleData => {
    const worst = [...(issues.get(n.id) ?? [])].sort((a, b) => Number(b.severity === "error") - Number(a.severity === "error"))[0];
    const error = worst?.severity === "error";
    const sub = hint(n, children.get(n.id)?.length ?? 0);
    const chip = worst ? RULE_CHIP[worst.rule] ?? "!" : undefined;
    return {
      variant: "item",
      label: n.kind === "plugin" ? `${expanded.has(n.id) ? "⌄" : "›"}  ${n.name}` : n.name,
      sub,
      chip,
      chipColor: error ? COLORS.error : COLORS.warning,
      dashed: error,
      color: error ? COLORS.error : groupOf(n.kind).color,
      scope: n.scope,
      disabled: !n.enabled,
      selected: n.id === selected,
      target: n.id,
      ...itemSize(n.name, sub, chip),
    };
  };

  const overlaps = (p: { x: number; y: number }, d: PebbleData) =>
    nodes.some((o) => Math.abs(o.position.x - p.x) < (o.data.w + d.w) / 2 + GAP && Math.abs(o.position.y - p.y) < (o.data.h + d.h) / 2 + GAP);

  /**
   * Places pebbles in columns off `origin` and links them to `from`. Returns their positions.
   * A pebble landing on one already placed slides outward along its branch until it's free.
   */
  // ponytail: checks every placed pebble, O(n²); a spatial grid if graphs reach thousands of pebbles.
  const fan = (datas: PebbleData[], origin: { x: number; y: number }, angle: number, from: string) => {
    const [dx, dy] = [Math.cos(rad(angle)), Math.sin(rad(angle))];
    return column(origin, angle, datas.length, Math.max(0, ...datas.map((d) => d.w))).map((p, i) => {
      const d = datas[i];
      while (overlaps(p, d)) p = { x: p.x + dx * 12, y: p.y + dy * 12 };
      const id = d.variant === "more" ? `more:${d.target}` : d.target;
      nodes.push(pebble(id, p, d));
      edges.push(edge(from, id, d.color, d.dashed));
      return p;
    });
  };

  for (const g of GROUPS) {
    if (hidden.has(g.key)) continue;
    const gid = `group:${g.key}`;
    const gpos = { x: Math.cos(rad(g.angle)) * GROUP_RADIUS, y: Math.sin(rad(g.angle)) * GROUP_RADIUS };
    const items = graph.nodes
      .filter((n) => !n.parent && g.kinds.includes(n.kind) && inScope(n))
      .sort((a, b) => Number(issues.has(b.id)) - Number(issues.has(a.id)) || a.name.localeCompare(b.name));
    nodes.push(pebble(gid, gpos, { variant: "group", label: g.label, sub: `${items.length}`, color: g.color, target: g.key, selected: gid === selected, dim: !!scope && !items.length, w: 82, h: 82 }));
    edges.push({ ...edge("center", gid, g.color), data: { color: g.color, dim: !!scope && !items.length } });

    const collapsible = items.length > COLLAPSED_MAX + 1;
    const open = expanded.has(g.key) || !collapsible;
    const shown = open ? items : items.slice(0, COLLAPSED_MAX);
    const datas = shown.map(itemData);
    if (collapsible) {
      const label = open ? t("‹ collapse") : t("+ {0} more", items.length - shown.length);
      datas.push({ variant: "more", label, color: g.color, dashed: true, target: g.key, w: 124, h: 40 });
    }
    const slots = fan(datas, gpos, g.angle, gid);

    // Expanded plugins fan their content further out, away from the center.
    shown.forEach((p, i) => {
      if (p.kind !== "plugin" || !expanded.has(p.id)) return;
      const angle = (Math.atan2(slots[i].y, slots[i].x) * 180) / Math.PI;
      fan((children.get(p.id) ?? []).map(itemData), slots[i], angle, p.id);
    });
  }

  const placed = new Set(nodes.map((n) => n.id));
  for (const i of graph.issues.filter((i) => i.rule === "duplicate")) {
    for (const other of i.nodes.slice(1)) {
      if (placed.has(i.nodes[0]) && placed.has(other)) edges.push(edge(i.nodes[0], other, COLORS.warning, true));
    }
  }
  if (selected) focus(nodes, edges, selected);
  return { nodes, edges };
}

/** Lights the selected pebble, its whole branch below it and the link to its parent; dims the rest. */
function focus(nodes: Node<PebbleData>[], edges: Edge<RoughEdgeData>[], selected: string) {
  const branch = new Set([selected]);
  for (let grew = true; grew; ) {
    grew = false;
    for (const e of edges) {
      if (branch.has(e.source) && !branch.has(e.target)) grew = !!branch.add(e.target);
    }
  }
  const related = new Set(branch);
  for (const e of edges) {
    const active = (branch.has(e.source) && branch.has(e.target)) || e.target === selected;
    if (active) related.add(e.source);
    e.data = { ...e.data!, active, dim: !active };
  }
  for (const n of nodes) n.data.dim = !related.has(n.id);
}

function pebble(id: string, position: { x: number; y: number }, data: PebbleData): Node<PebbleData> {
  return { id, type: "pebble", position, data, width: data.w, height: data.h };
}

function edge(source: string, target: string, color: string, dashed = false): Edge<RoughEdgeData> {
  return { id: `${source}->${target}`, source, target, type: "rough", data: { color, dashed } };
}
