import { useEffect, useMemo } from "react";
import { ReactFlow, useReactFlow, type NodeMouseHandler } from "@xyflow/react";
import { t } from "../i18n";
import { layout, type PebbleData } from "../layout";
import type { GroupKey } from "../theme";
import type { Graph, Scope } from "../types";
import { PebbleNode, RoughEdge } from "./Pebble";

const nodeTypes = { pebble: PebbleNode };
const edgeTypes = { rough: RoughEdge };

type Props = {
  graph: Graph;
  hidden: Set<GroupKey>;
  expanded: Set<string>;
  selected: string | null;
  scope: Scope | null;
  onSelect: (id: string | null) => void;
  onToggle: (key: string) => void;
};

export function GraphView({ graph, hidden, expanded, selected, scope, onSelect, onToggle }: Props) {
  const { nodes, edges } = useMemo(() => layout(graph, hidden, expanded, selected, scope), [graph, hidden, expanded, selected, scope]);
  const { fitView, zoomIn, zoomOut } = useReactFlow();

  // Refit only when the account/project changes, not on every rescan.
  useEffect(() => {
    requestAnimationFrame(() => fitView({ padding: 0.1 }));
  }, [graph.config_dir, graph.project, fitView]);

  const onNodeClick: NodeMouseHandler = (_, node) => {
    const d = node.data as PebbleData;
    if (d.variant === "more") return onToggle(d.target);
    if (d.variant === "group") return onSelect(node.id);
    if (d.variant === "item") {
      if (graph.nodes.find((n) => n.id === d.target)?.kind === "plugin") onToggle(d.target);
      onSelect(d.target);
    }
  };

  return (
    <div className="canvas">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        edgeTypes={edgeTypes}
        nodeOrigin={[0.5, 0.5]}
        nodesDraggable={false}
        nodesConnectable={false}
        elementsSelectable={false}
        onNodeClick={onNodeClick}
        onPaneClick={() => onSelect(null)}
        minZoom={0.2}
        fitView
        proOptions={{ hideAttribution: true }}
      />
      <div className="zoom">
        <button onClick={() => zoomIn()} aria-label={t("Zoom in")}>+</button>
        <button onClick={() => zoomOut()} aria-label={t("Zoom out")}>−</button>
      </div>
    </div>
  );
}
