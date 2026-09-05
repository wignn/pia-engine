const nodes = {
  sources: { x: 470, y: 94, w: 460, h: 112, kind: "source", title: "EXTERNAL SOURCES", lines: ["Market feeds · RSS / news · Calendar", "RSSHub / social · Analyzer / LLM providers"] },
  ingest: { x: 90, y: 292, w: 350, h: 112, kind: "service", title: "INGESTION GATEWAY", lines: ["Vendor sessions · normalization", "Event publishing"] },
  nats: { x: 525, y: 292, w: 350, h: 112, kind: "store", title: "NATS JETSTREAM", lines: ["Durable domain events"] },
  redis: { x: 960, y: 292, w: 350, h: 112, kind: "store", title: "REDIS", lines: ["Cache · counters", "Compatibility pub/sub"] },
  market: { x: 20, y: 530, w: 370, h: 116, kind: "service", title: "MARKET DATA", lines: ["Prices · history · quality", "Spikes"] },
  news: { x: 515, y: 530, w: 370, h: 116, kind: "service", title: "NEWS SERVICE", lines: ["Forex / news · stock / news", "Calendar · source status"] },
  intel: { x: 1010, y: 530, w: 370, h: 116, kind: "accent", title: "INTELLIGENCE SERVICE", lines: ["Analyze · sentiment", "Why Did It Move · factors"] },
  pg: { x: 210, y: 764, w: 350, h: 112, kind: "store", title: "POSTGRESQL", lines: ["SaaS state · news", "Cache rows"] },
  ch: { x: 615, y: 764, w: 350, h: 112, kind: "store", title: "CLICKHOUSE", lines: ["Ticks · history", "OHLC reads"] },
  analyzer: { x: 1020, y: 764, w: 350, h: 112, kind: "accent", title: "ANALYZER", lines: ["FinBERT · language", "Optional LLM hooks"] },
  api: { x: 300, y: 1010, w: 370, h: 116, kind: "service", title: "API GATEWAY", lines: ["REST entrypoint · API keys", "Quota · proxy"] },
  realtime: { x: 730, y: 1010, w: 370, h: 116, kind: "service", title: "REALTIME GATEWAY", lines: ["WebSocket fanout", "Subscriptions · tickets"] },
  clients: { x: 315, y: 1288, w: 770, h: 112, kind: "client", title: "CLIENT SURFACES", lines: ["Public web · Admin web · Desktop trader", "Discord bot · Public API clients"] },
} as const;

type NodeId = keyof typeof nodes;
type Node = (typeof nodes)[NodeId];

type Edge = {
  from: NodeId;
  to: NodeId;
  label?: string;
  dashed?: boolean;
  fromSide?: "top" | "right" | "bottom" | "left";
  toSide?: "top" | "right" | "bottom" | "left";
};

const edges: Edge[] = [
  { from: "sources", to: "ingest", label: "connect / normalize", fromSide: "bottom", toSide: "top" },
  { from: "ingest", to: "nats", label: "publish events", fromSide: "right", toSide: "left" },
  { from: "ingest", to: "redis", label: "cache / counters", fromSide: "right", toSide: "left" },
  { from: "nats", to: "market", label: "ticks", fromSide: "bottom", toSide: "top" },
  { from: "nats", to: "news", label: "news / calendar", fromSide: "bottom", toSide: "top" },
  { from: "nats", to: "intel", label: "domain events", fromSide: "bottom", toSide: "top" },
  { from: "redis", to: "realtime", label: "transitional pub/sub", dashed: true, fromSide: "bottom", toSide: "top" },
  { from: "market", to: "pg", label: "state", fromSide: "bottom", toSide: "top" },
  { from: "market", to: "ch", label: "time series", fromSide: "bottom", toSide: "top" },
  { from: "news", to: "pg", label: "archive", fromSide: "bottom", toSide: "top" },
  { from: "intel", to: "pg", label: "factors", fromSide: "bottom", toSide: "top" },
  { from: "intel", to: "ch", label: "reads", fromSide: "bottom", toSide: "top" },
  { from: "intel", to: "analyzer", label: "analyze text", fromSide: "right", toSide: "left" },
  { from: "api", to: "market", label: "query", fromSide: "top", toSide: "bottom" },
  { from: "api", to: "news", label: "query", fromSide: "top", toSide: "bottom" },
  { from: "api", to: "intel", label: "query", fromSide: "top", toSide: "bottom" },
  { from: "api", to: "clients", label: "REST output", fromSide: "bottom", toSide: "top" },
  { from: "realtime", to: "clients", label: "WebSocket output", fromSide: "bottom", toSide: "top" },
];

const anchor = (node: Node, side: NonNullable<Edge["fromSide"]>) => {
  if (side === "top") return { x: node.x + node.w / 2, y: node.y };
  if (side === "right") return { x: node.x + node.w, y: node.y + node.h / 2 };
  if (side === "left") return { x: node.x, y: node.y + node.h / 2 };
  return { x: node.x + node.w / 2, y: node.y + node.h };
};

function connectorPath(edge: Edge) {
  const from = anchor(nodes[edge.from], edge.fromSide ?? "bottom");
  const to = anchor(nodes[edge.to], edge.toSide ?? "top");
  const horizontal = edge.fromSide === "right" || edge.fromSide === "left";
  if (horizontal) {
    const midX = (from.x + to.x) / 2;
    return `M ${from.x} ${from.y} H ${midX} V ${to.y} H ${to.x}`;
  }
  const midY = (from.y + to.y) / 2;
  return `M ${from.x} ${from.y} V ${midY} H ${to.x} V ${to.y}`;
}

function DiagramNode({ node }: { node: Node }) {
  return (
    <g className={`architecture-diagram-node is-${node.kind}`}>
      <rect x={node.x} y={node.y} width={node.w} height={node.h} rx="4" />
      <text className="architecture-diagram-node-title" x={node.x + 22} y={node.y + 34}>{node.title}</text>
      <text className="architecture-diagram-node-detail" x={node.x + 22} y={node.y + 65}>{node.lines[0]}</text>
      {node.lines[1] && <text className="architecture-diagram-node-detail" x={node.x + 22} y={node.y + 87}>{node.lines[1]}</text>}
    </g>
  );
}

type ArchitectureDiagramProps = {
  embedded?: boolean;
};

export function ArchitectureDiagram({ embedded = false }: ArchitectureDiagramProps) {
  return (
    <section className={`architecture-page${embedded ? " architecture-page-embedded" : ""}`} id="architecture-map">
      <header className="architecture-page-header">
        <a className="architecture-back" href={embedded ? "#top" : "/portal"}>{embedded ? "↘ RETURN TO SIGNAL" : "← SLV PORTAL"}</a>
        <div className="architecture-header-meta">ATLSD ENGINE / SYSTEM MAP / V1.0</div>
      </header>

      <section className="architecture-intro" aria-labelledby="architecture-page-title">
        <p className="architecture-kicker">SYSTEM TOPOLOGY · TOP TO BOTTOM</p>
        <h1 id="architecture-page-title">THE SIGNAL<br /><em>THROUGH THE SYSTEM.</em></h1>
        {/* <p>How external market signals are normalized, distributed through domain services, persisted for analysis, and delivered through public gateways.</p> */}
      </section>

      <figure className="architecture-figure">
        <div className="architecture-diagram-frame">
          <svg className="architecture-diagram" viewBox="0 0 1400 1500" role="img" aria-labelledby="architecture-diagram-title architecture-diagram-description">
            <title id="architecture-diagram-title">ATLSD Engine top-to-bottom architecture flowchart</title>
            <desc id="architecture-diagram-description">External sources flow into ingestion, durable events and cache, then domain services, persistence and analyzer services, public gateways, and client surfaces.</desc>
            <defs>
              <marker id="architecture-arrow" markerWidth="8" markerHeight="8" refX="7" refY="4" orient="auto" markerUnits="strokeWidth"><path d="M0,0 L8,4 L0,8 Z" /></marker>
            </defs>
            <g className="architecture-band-labels" aria-hidden="true">
              <text x="20" y="54">01 / SOURCES</text><text x="20" y="258">02 / INGESTION + EVENTS</text><text x="20" y="496">03 / DOMAIN SERVICES</text><text x="20" y="730">04 / PERSISTENCE + ANALYSIS</text><text x="20" y="976">05 / PUBLIC GATEWAYS</text><text x="20" y="1254">06 / CLIENT SURFACES</text>
            </g>
            <g className="architecture-connectors" aria-hidden="true">
              {edges.map((edge) => <g key={`${edge.from}-${edge.to}`}><path className={edge.dashed ? "is-dashed" : ""} d={connectorPath(edge)} markerEnd="url(#architecture-arrow)" /><text className="architecture-edge-label" x={(anchor(nodes[edge.from], edge.fromSide ?? "bottom").x + anchor(nodes[edge.to], edge.toSide ?? "top").x) / 2} y={(anchor(nodes[edge.from], edge.fromSide ?? "bottom").y + anchor(nodes[edge.to], edge.toSide ?? "top").y) / 2 - 7}>{edge.label}</text></g>)}
            </g>
            <g>{(Object.keys(nodes) as NodeId[]).map((id) => <DiagramNode node={nodes[id]} key={id} />)}</g>
          </svg>
        </div>
        {/* <figcaption className="architecture-caption">
          <span><i className="legend-line" /> primary data flow</span>
          <span><i className="legend-line is-dashed" /> transitional compatibility path</span>
          <span><i className="legend-node is-service" /> service</span>
          <span><i className="legend-node is-store" /> durable store</span>
          <p>The diagram is a public system overview. It describes product-level boundaries, not private deployment configuration or live infrastructure health.</p>
        </figcaption> */}
      </figure>
    </section>
  );
}

export function ArchitectureDiagramPage() {
  return <ArchitectureDiagram />;
}
