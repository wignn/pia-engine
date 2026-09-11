import json

def build_dashboard():
    with open('infra/monitoring/grafana/dashboards/atlsd-production-overview.json') as f:
        data = json.load(f)

    panels = []
    pid = 1

    def make_stat_service(title, service_name, x, y, w=3, h=3):
        nonlocal pid
        panel = {
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "fieldConfig": {
                "defaults": {
                    "color": {"mode": "thresholds"},
                    "mappings": [
                        {
                            "options": {
                                "0": {"color": "red", "index": 0, "text": "DOWN"},
                                "1": {"color": "green", "index": 1, "text": "UP"}
                            },
                            "type": "value"
                        }
                    ],
                    "thresholds": {
                        "mode": "absolute",
                        "steps": [
                            {"color": "red", "value": None},
                            {"color": "green", "value": 1}
                        ]
                    },
                    "unit": "short"
                },
                "overrides": []
            },
            "gridPos": {"h": h, "w": w, "x": x, "y": y},
            "id": pid,
            "options": {
                "colorMode": "background",
                "graphMode": "none",
                "justifyMode": "center",
                "orientation": "horizontal",
                "reduceOptions": {
                    "calcs": ["lastNotNull"],
                    "fields": "",
                    "values": False
                },
                "showPercentChange": False,
                "textMode": "auto",
                "wideLayout": True
            },
            "pluginVersion": "11.3.1",
            "targets": [
                {
                    "datasource": {"type": "prometheus", "uid": "prometheus"},
                    "editorMode": "code",
                    "expr": f'max(probe_success{{service="{service_name}"}}) or vector(0)',
                    "instant": True,
                    "legendFormat": "",
                    "refId": "A"
                }
            ],
            "title": title,
            "type": "stat"
        }
        pid += 1
        return panel

    # -------------------------------------------------------------
    # ROW 1: EXECUTIVE PLATFORM HUD (y=0, panels y=1, h=3)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 0},
        "id": pid,
        "title": "EXECUTIVE PLATFORM HUD",
        "type": "row"
    })
    pid += 1

    # Panel 1: Down Services (x=0, w=4)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "thresholds"},
                "mappings": [
                    {
                        "options": {
                            "0": {"color": "green", "index": 0, "text": "0 (ALL UP)"}
                        },
                        "type": "value"
                    }
                ],
                "thresholds": {
                    "mode": "absolute",
                    "steps": [
                        {"color": "green", "value": None},
                        {"color": "yellow", "value": 1},
                        {"color": "red", "value": 2}
                    ]
                },
                "unit": "short"
            },
            "overrides": []
        },
        "gridPos": {"h": 3, "w": 4, "x": 0, "y": 1},
        "id": pid,
        "options": {
            "colorMode": "background",
            "graphMode": "none",
            "justifyMode": "center",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False},
            "showPercentChange": False,
            "textMode": "auto",
            "wideLayout": True
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": 'sum(probe_success{group=~"atlsd-services|atlsd-datastores"} == bool 0)',
            "instant": True,
            "legendFormat": "",
            "refId": "A"
        }],
        "title": "Down Services (Now)",
        "type": "stat"
    })
    pid += 1

    # Panel 2: Active WS (x=4, w=4)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "thresholds"},
                "mappings": [],
                "thresholds": {
                    "mode": "absolute",
                    "steps": [{"color": "blue", "value": None}, {"color": "green", "value": 1}]
                },
                "unit": "short"
            },
            "overrides": []
        },
        "gridPos": {"h": 3, "w": 4, "x": 4, "y": 1},
        "id": pid,
        "options": {
            "colorMode": "value",
            "graphMode": "area",
            "justifyMode": "center",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False},
            "textMode": "auto"
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": 'max(atlsd_realtime_ws_active_connections{job="realtime-gateway"}) or vector(0)',
            "instant": False,
            "range": True,
            "refId": "A"
        }],
        "title": "Active WebSocket Sessions",
        "type": "stat"
    })
    pid += 1

    # Panel 3: WS Out /s (x=8, w=4)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "thresholds"},
                "mappings": [],
                "thresholds": {
                    "mode": "absolute",
                    "steps": [{"color": "green", "value": None}]
                },
                "unit": "reqps"
            },
            "overrides": []
        },
        "gridPos": {"h": 3, "w": 4, "x": 8, "y": 1},
        "id": pid,
        "options": {
            "colorMode": "value",
            "graphMode": "area",
            "justifyMode": "center",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False},
            "textMode": "auto"
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": 'sum(rate(atlsd_realtime_ws_messages_out_total{job="realtime-gateway"}[5m])) or vector(0)',
            "instant": False,
            "range": True,
            "refId": "A"
        }],
        "title": "WS Broadcast Outbound /s",
        "type": "stat"
    })
    pid += 1

    # Panel 4: Backpressure / Send Failures (x=12, w=4)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "thresholds"},
                "mappings": [],
                "thresholds": {
                    "mode": "absolute",
                    "steps": [{"color": "green", "value": None}, {"color": "red", "value": 0.01}]
                },
                "unit": "reqps"
            },
            "overrides": []
        },
        "gridPos": {"h": 3, "w": 4, "x": 12, "y": 1},
        "id": pid,
        "options": {
            "colorMode": "value",
            "graphMode": "none",
            "justifyMode": "center",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False},
            "textMode": "auto"
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": 'sum(rate(atlsd_realtime_ws_send_failures_total{job="realtime-gateway"}[5m])) or vector(0)',
            "instant": False,
            "range": True,
            "refId": "A"
        }],
        "title": "WS Backpressure /s",
        "type": "stat"
    })
    pid += 1

    # Panel 5: CPU % (x=16, w=4)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "thresholds"},
                "mappings": [],
                "thresholds": {
                    "mode": "absolute",
                    "steps": [
                        {"color": "green", "value": None},
                        {"color": "yellow", "value": 70},
                        {"color": "red", "value": 85}
                    ]
                },
                "unit": "percent",
                "max": 100,
                "min": 0
            },
            "overrides": []
        },
        "gridPos": {"h": 3, "w": 4, "x": 16, "y": 1},
        "id": pid,
        "options": {
            "colorMode": "value",
            "graphMode": "area",
            "justifyMode": "center",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False},
            "textMode": "auto"
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": "100 - (avg(rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)",
            "range": True,
            "refId": "A"
        }],
        "title": "Host CPU Utilization",
        "type": "stat"
    })
    pid += 1

    # Panel 6: Memory % (x=20, w=4)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "thresholds"},
                "mappings": [],
                "thresholds": {
                    "mode": "absolute",
                    "steps": [
                        {"color": "green", "value": None},
                        {"color": "yellow", "value": 80},
                        {"color": "red", "value": 90}
                    ]
                },
                "unit": "percent",
                "max": 100,
                "min": 0
            },
            "overrides": []
        },
        "gridPos": {"h": 3, "w": 4, "x": 20, "y": 1},
        "id": pid,
        "options": {
            "colorMode": "value",
            "graphMode": "area",
            "justifyMode": "center",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False},
            "textMode": "auto"
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": "(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100",
            "range": True,
            "refId": "A"
        }],
        "title": "Host Memory Utilization",
        "type": "stat"
    })
    pid += 1

    # -------------------------------------------------------------
    # ROW 2: CORE SAAS SERVICES LIVENESS (y=4, panels y=5, h=3)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 4},
        "id": pid,
        "title": "CORE SAAS APPLICATIONS & PLATFORM NODES (LIVE MATRIX)",
        "type": "row"
    })
    pid += 1

    services = [
        ("api-gateway", "api-gateway"),
        ("realtime-ws", "realtime-gateway"),
        ("control-plane", "control-plane"),
        ("ingestion-gw", "ingestion-gateway"),
        ("pia-portal", "pia-portal"),
        ("mission-control", "mission-control"),
        ("public-web", "public-web"),
        ("terminal-app", "terminal"),
    ]

    for idx, (title, name) in enumerate(services):
        x = idx * 3
        panels.append(make_stat_service(title, name, x, 5, w=3, h=3))

    # -------------------------------------------------------------
    # ROW 3: AVAILABILITY & LATENCY SLA (y=8, panels y=9, h=8)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 8},
        "id": pid,
        "title": "AVAILABILITY TIMELINE & LATENCY SLA (CORE APPLICATIONS)",
        "type": "row"
    })
    pid += 1

    # Left: Availability history (w=15, h=8) - Clean 8 Core Services
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "thresholds"},
                "custom": {"fillOpacity": 80, "lineWidth": 1},
                "mappings": [
                    {
                        "options": {
                            "0": {"color": "red", "index": 0, "text": "DOWN"},
                            "1": {"color": "green", "index": 1, "text": "UP"}
                        },
                        "type": "value"
                    }
                ],
                "thresholds": {
                    "mode": "absolute",
                    "steps": [{"color": "red", "value": None}, {"color": "green", "value": 1}]
                }
            },
            "overrides": []
        },
        "gridPos": {"h": 8, "w": 15, "x": 0, "y": 9},
        "id": pid,
        "options": {
            "showValue": "never",
            "rowHeight": 0.65
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": 'max by (service) (last_over_time(probe_success{service=~"api-gateway|realtime-gateway|control-plane|ingestion-gateway|pia-portal|mission-control|public-web|terminal"}[$__interval]))',
            "legendFormat": "{{service}}",
            "range": True,
            "interval": "2m",
            "refId": "A"
        }],
        "title": "Service Availability History (Rolling)",
        "type": "status-history"
    })
    pid += 1

    # Right: Latency Ranking Bar Gauge (w=9, h=8) - Same 8 Core Services
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "thresholds"},
                "mappings": [],
                "thresholds": {
                    "mode": "absolute",
                    "steps": [
                        {"color": "green", "value": None},
                        {"color": "yellow", "value": 50},
                        {"color": "red", "value": 200}
                    ]
                },
                "unit": "ms"
            },
            "overrides": []
        },
        "gridPos": {"h": 8, "w": 9, "x": 15, "y": 9},
        "id": pid,
        "options": {
            "displayMode": "gradient",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False},
            "showUnfilled": True
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": 'avg_over_time(probe_duration_seconds{service=~"api-gateway|realtime-gateway|control-plane|ingestion-gateway|pia-portal|mission-control|public-web|terminal"}[1m]) * 1000',
            "legendFormat": "{{service}}",
            "range": True,
            "refId": "A"
        }],
        "title": "Service Response Latency (ms)",
        "type": "bargauge"
    })
    pid += 1

    # -------------------------------------------------------------
    # ROW 4: REALTIME WEBSOCKET STREAMING (y=17, panels y=18, h=6)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 17},
        "id": pid,
        "title": "REALTIME WEBSOCKET STREAMING (SUBSCRIBERS & THROUGHPUT)",
        "type": "row"
    })
    pid += 1

    # Left: WS Throughput (w=12) - strictly scoped to realtime-gateway job
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "custom": {
                    "drawStyle": "line",
                    "lineInterpolation": "smooth",
                    "lineWidth": 2,
                    "fillOpacity": 12
                },
                "unit": "reqps"
            },
            "overrides": []
        },
        "gridPos": {"h": 6, "w": 12, "x": 0, "y": 18},
        "id": pid,
        "options": {"legend": {"displayMode": "table", "placement": "bottom", "calcs": ["last", "max"]}},
        "targets": [
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": 'rate(atlsd_realtime_ws_messages_out_total{job="realtime-gateway"}[5m])',
                "legendFormat": "Outbound Messages /s",
                "refId": "A"
            },
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": 'rate(atlsd_realtime_ws_messages_in_total{job="realtime-gateway"}[5m])',
                "legendFormat": "Inbound Messages /s",
                "refId": "B"
            },
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": 'rate(atlsd_realtime_ws_commands_total{job="realtime-gateway"}[5m])',
                "legendFormat": "Client Commands /s",
                "refId": "C"
            }
        ],
        "title": "WebSocket Message Flow Rate",
        "type": "timeseries"
    })
    pid += 1

    # Right: WS Health (w=12) - strictly scoped to realtime-gateway job
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "custom": {
                    "drawStyle": "line",
                    "lineInterpolation": "smooth",
                    "lineWidth": 2,
                    "fillOpacity": 8
                },
                "unit": "reqps"
            },
            "overrides": []
        },
        "gridPos": {"h": 6, "w": 12, "x": 12, "y": 18},
        "id": pid,
        "options": {"legend": {"displayMode": "table", "placement": "bottom", "calcs": ["last", "max"]}},
        "targets": [
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": 'rate(atlsd_realtime_ws_broadcast_recipients_total{job="realtime-gateway"}[5m])',
                "legendFormat": "Broadcast Fanout /s",
                "refId": "A"
            },
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": 'rate(atlsd_realtime_ws_send_failures_total{job="realtime-gateway"}[5m])',
                "legendFormat": "Send Failures / Backpressure",
                "refId": "B"
            },
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": 'rate(atlsd_realtime_ws_connection_rejections_total{job="realtime-gateway"}[5m])',
                "legendFormat": "Connection Rejections /s",
                "refId": "C"
            }
        ],
        "title": "Fanout Quality & Delivery Health",
        "type": "timeseries"
    })
    pid += 1

    # -------------------------------------------------------------
    # ROW 5: HOST INFRASTRUCTURE METRICS (y=24, panels y=25, h=5)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 24},
        "id": pid,
        "title": "HOST INFRASTRUCTURE METRICS (CPU / RAM / DISK)",
        "type": "row"
    })
    pid += 1

    # CPU (w=8)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "custom": {"drawStyle": "line", "lineInterpolation": "smooth", "fillOpacity": 15},
                "unit": "percent", "min": 0, "max": 100
            },
            "overrides": []
        },
        "gridPos": {"h": 5, "w": 8, "x": 0, "y": 25},
        "id": pid,
        "options": {"legend": {"displayMode": "list", "placement": "bottom"}},
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "expr": "100 - (avg(rate(node_cpu_seconds_total{mode=\"idle\"}[5m])) * 100)",
            "legendFormat": "Host CPU %",
            "refId": "A"
        }],
        "title": "Host CPU Trend",
        "type": "timeseries"
    })
    pid += 1

    # Memory (w=8)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "custom": {"drawStyle": "line", "lineInterpolation": "smooth", "fillOpacity": 15},
                "unit": "percent", "min": 0, "max": 100
            },
            "overrides": []
        },
        "gridPos": {"h": 5, "w": 8, "x": 8, "y": 25},
        "id": pid,
        "options": {"legend": {"displayMode": "list", "placement": "bottom"}},
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "expr": "(1 - (node_memory_MemAvailable_bytes / node_memory_MemTotal_bytes)) * 100",
            "legendFormat": "Host RAM %",
            "refId": "A"
        }],
        "title": "Host Memory Trend",
        "type": "timeseries"
    })
    pid += 1

    # Disk (w=8) - single clean line for rootfs!
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "custom": {"drawStyle": "line", "lineInterpolation": "smooth", "fillOpacity": 15},
                "unit": "percent", "min": 0, "max": 100
            },
            "overrides": []
        },
        "gridPos": {"h": 5, "w": 8, "x": 16, "y": 25},
        "id": pid,
        "options": {"legend": {"displayMode": "list", "placement": "bottom"}},
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "expr": "100 - ((node_filesystem_avail_bytes{mountpoint=\"/rootfs\"} * 100) / node_filesystem_size_bytes{mountpoint=\"/rootfs\"})",
            "legendFormat": "Root Disk (/) Used %",
            "refId": "A"
        }],
        "title": "Root Filesystem Usage %",
        "type": "timeseries"
    })
    pid += 1

    # -------------------------------------------------------------
    # ROW 6: CONTAINER RESOURCE CONSUMPTION & AUDIT LOGS (y=30, panels y=31, h=6)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 30},
        "id": pid,
        "title": "CONTAINER RESOURCE CONSUMPTION & LOG INGESTION",
        "type": "row"
    })
    pid += 1

    # Container CPU (w=7) - uses atlsd_docker_container_cpu_percent
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "palette-classic"},
                "unit": "percent"
            },
            "overrides": []
        },
        "gridPos": {"h": 6, "w": 7, "x": 0, "y": 31},
        "id": pid,
        "options": {
            "displayMode": "gradient",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False}
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "expr": 'topk(5, atlsd_docker_container_cpu_percent)',
            "legendFormat": "{{container}}",
            "refId": "A"
        }],
        "title": "Top Container CPU %",
        "type": "bargauge"
    })
    pid += 1

    # Container Memory (w=7) - uses atlsd_docker_container_memory_usage_bytes
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "palette-classic"},
                "unit": "bytes"
            },
            "overrides": []
        },
        "gridPos": {"h": 6, "w": 7, "x": 7, "y": 31},
        "id": pid,
        "options": {
            "displayMode": "gradient",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False}
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "expr": 'topk(5, atlsd_docker_container_memory_usage_bytes)',
            "legendFormat": "{{container}}",
            "refId": "A"
        }],
        "title": "Top Container RAM (Bytes)",
        "type": "bargauge"
    })
    pid += 1

    # Loki Log Ingestion Volume Rate (w=10) - uses real container labels in Loki!
    panels.append({
        "datasource": {"type": "loki", "uid": "loki"},
        "fieldConfig": {
            "defaults": {
                "custom": {"drawStyle": "bars", "fillOpacity": 40},
                "unit": "ops"
            },
            "overrides": []
        },
        "gridPos": {"h": 6, "w": 10, "x": 14, "y": 31},
        "id": pid,
        "options": {"legend": {"displayMode": "list", "placement": "bottom"}},
        "targets": [{
            "datasource": {"type": "loki", "uid": "loki"},
            "editorMode": "code",
            "expr": 'sum by (container) (rate({container=~".+"} [1m]))',
            "legendFormat": "{{container}}",
            "refId": "A"
        }],
        "title": "Container Log Volume Rate (/s)",
        "type": "timeseries"
    })
    pid += 1

    data["panels"] = panels
    data["title"] = "ATLSD Production Command Center"
    data["version"] = data.get("version", 1) + 1

    with open('infra/monitoring/grafana/dashboards/atlsd-production-overview.json', 'w') as f:
        json.dump(data, f, indent=2)

    print(f"Successfully generated perfected dashboard with {len(panels)} panels!")

if __name__ == '__main__':
    build_dashboard()
