import json

def build_dashboard():
    with open('infra/monitoring/grafana/dashboards/atlsd-production-overview.json') as f:
        data = json.load(f)

    # Let's rebuild the panels array with strict mathematical grid layout
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
                                "1": {"color": "green", "index": 1, "text": "HEALTHY"}
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
                    "expr": f'probe_success{{group=~"atlsd-services|atlsd-datastores",service="{service_name}"}}',
                    "instant": False,
                    "legendFormat": "",
                    "range": True,
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
            "expr": 'sum(min_over_time(probe_success{group=~"atlsd-services|atlsd-datastores",job=~"service-health|datastore-tcp"}[5m]) == bool 0)',
            "instant": False,
            "legendFormat": "",
            "range": True,
            "refId": "A"
        }],
        "title": "Down Services (5m)",
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
            "expr": "atlsd_realtime_ws_active_connections",
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
            "expr": "rate(atlsd_realtime_ws_messages_out_total[5m])",
            "range": True,
            "refId": "A"
        }],
        "title": "WS Broadcast Throughput",
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
            "expr": "rate(atlsd_realtime_ws_send_failures_total[5m])",
            "range": True,
            "refId": "A"
        }],
        "title": "Backpressure Drops /s",
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
        "title": "CORE SAAS MICROSERVICES & APPS (LIVE MATRIX)",
        "type": "row"
    })
    pid += 1

    services = [
        ("api-gateway", "api-gateway"),
        ("realtime-gateway", "realtime-gateway"),
        ("market-data", "market-data"),
        ("news-service", "news-service"),
        ("intelligence", "intelligence-service"),
        ("control-plane", "world-control-plane"),
        ("pia-portal", "pia-portal"),
        ("mission-control", "mission-control"),
    ]

    for idx, (title, name) in enumerate(services):
        x = idx * 3
        panels.append(make_stat_service(title, name, x, 5, w=3, h=3))

    # -------------------------------------------------------------
    # ROW 3: AVAILABILITY & LATENCY SLA (y=8, panels y=9, h=6)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 8},
        "id": pid,
        "title": "AVAILABILITY TIMELINE & LATENCY SLA",
        "type": "row"
    })
    pid += 1

    # Left: Availability history (w=15)
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
        "gridPos": {"h": 6, "w": 15, "x": 0, "y": 9},
        "id": pid,
        "options": {"showValue": "never"},
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "editorMode": "code",
            "expr": "max by (service) (probe_success)",
            "legendFormat": "{{service}}",
            "range": True,
            "refId": "A"
        }],
        "title": "Service Availability History (Rolling)",
        "type": "status-history"
    })
    pid += 1

    # Right: Latency Ranking Bar Gauge (w=9)
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
        "gridPos": {"h": 6, "w": 9, "x": 15, "y": 9},
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
            "expr": 'avg_over_time(probe_duration_seconds{group=~"atlsd-services|atlsd-datastores"}[1m]) * 1000',
            "legendFormat": "{{service}}",
            "range": True,
            "refId": "A"
        }],
        "title": "Service Response Latency (ms)",
        "type": "bargauge"
    })
    pid += 1

    # -------------------------------------------------------------
    # ROW 4: REALTIME WEBSOCKET STREAMING (y=15, panels y=16, h=6)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 15},
        "id": pid,
        "title": "REALTIME WEBSOCKET STREAMING (SUBSCRIBERS & THROUGHPUT)",
        "type": "row"
    })
    pid += 1

    # Left: WS Throughput (w=12)
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
        "gridPos": {"h": 6, "w": 12, "x": 0, "y": 16},
        "id": pid,
        "options": {"legend": {"displayMode": "table", "placement": "bottom", "calcs": ["last", "max"]}},
        "targets": [
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": "rate(atlsd_realtime_ws_messages_out_total[5m])",
                "legendFormat": "Outbound Messages /s",
                "refId": "A"
            },
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": "rate(atlsd_realtime_ws_messages_in_total[5m])",
                "legendFormat": "Inbound Messages /s",
                "refId": "B"
            },
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": "rate(atlsd_realtime_ws_commands_total[5m])",
                "legendFormat": "Client Commands /s",
                "refId": "C"
            }
        ],
        "title": "WebSocket Message Flow Rate",
        "type": "timeseries"
    })
    pid += 1

    # Right: WS Health (w=12)
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
        "gridPos": {"h": 6, "w": 12, "x": 12, "y": 16},
        "id": pid,
        "options": {"legend": {"displayMode": "table", "placement": "bottom", "calcs": ["last", "max"]}},
        "targets": [
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": "rate(atlsd_realtime_ws_broadcast_recipients_total[5m])",
                "legendFormat": "Broadcast Fanout Recipients /s",
                "refId": "A"
            },
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": "rate(atlsd_realtime_ws_send_failures_total[5m])",
                "legendFormat": "Send Failures / Backpressure",
                "refId": "B"
            },
            {
                "datasource": {"type": "prometheus", "uid": "prometheus"},
                "expr": "rate(atlsd_realtime_ws_connection_rejections_total[5m])",
                "legendFormat": "Connection Rejections /s",
                "refId": "C"
            }
        ],
        "title": "Fanout Quality & Backpressure",
        "type": "timeseries"
    })
    pid += 1

    # -------------------------------------------------------------
    # ROW 5: HOST INFRASTRUCTURE METRICS (y=22, panels y=23, h=5)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 22},
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
        "gridPos": {"h": 5, "w": 8, "x": 0, "y": 23},
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
        "gridPos": {"h": 5, "w": 8, "x": 8, "y": 23},
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

    # Disk (w=8)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "custom": {"drawStyle": "line", "lineInterpolation": "smooth", "fillOpacity": 15},
                "unit": "percent", "min": 0, "max": 100
            },
            "overrides": []
        },
        "gridPos": {"h": 5, "w": 8, "x": 16, "y": 23},
        "id": pid,
        "options": {"legend": {"displayMode": "list", "placement": "bottom"}},
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "expr": "100 - ((node_filesystem_avail_bytes{fstype!~\"tmpfs|overlay\"} * 100) / node_filesystem_size_bytes{fstype!~\"tmpfs|overlay\"})",
            "legendFormat": "Disk Used % ({{mountpoint}})",
            "refId": "A"
        }],
        "title": "Root Filesystem Usage %",
        "type": "timeseries"
    })
    pid += 1

    # -------------------------------------------------------------
    # ROW 6: CONTAINER RESOURCE CONSUMPTION & AUDIT LOGS (y=28, panels y=29, h=6)
    # -------------------------------------------------------------
    panels.append({
        "collapsed": False,
        "gridPos": {"h": 1, "w": 24, "x": 0, "y": 28},
        "id": pid,
        "title": "CONTAINER RESOURCE CONSUMPTION & AUDIT LOGS",
        "type": "row"
    })
    pid += 1

    # Container CPU (w=7)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "palette-classic"},
                "unit": "percent"
            },
            "overrides": []
        },
        "gridPos": {"h": 6, "w": 7, "x": 0, "y": 29},
        "id": pid,
        "options": {
            "displayMode": "gradient",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False}
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "expr": 'topk(5, rate(container_cpu_usage_seconds_total{image!="",name!=""}[5m]) * 100)',
            "legendFormat": "{{name}}",
            "refId": "A"
        }],
        "title": "Top Container CPU %",
        "type": "bargauge"
    })
    pid += 1

    # Container Memory (w=7)
    panels.append({
        "datasource": {"type": "prometheus", "uid": "prometheus"},
        "fieldConfig": {
            "defaults": {
                "color": {"mode": "palette-classic"},
                "unit": "bytes"
            },
            "overrides": []
        },
        "gridPos": {"h": 6, "w": 7, "x": 7, "y": 29},
        "id": pid,
        "options": {
            "displayMode": "gradient",
            "orientation": "horizontal",
            "reduceOptions": {"calcs": ["lastNotNull"], "fields": "", "values": False}
        },
        "targets": [{
            "datasource": {"type": "prometheus", "uid": "prometheus"},
            "expr": 'topk(5, container_memory_working_set_bytes{image!="",name!=""})',
            "legendFormat": "{{name}}",
            "refId": "A"
        }],
        "title": "Top Container RAM",
        "type": "bargauge"
    })
    pid += 1

    # Loki Logs Error Rate (w=10)
    panels.append({
        "datasource": {"type": "loki", "uid": "loki"},
        "fieldConfig": {
            "defaults": {
                "custom": {"drawStyle": "bars", "fillOpacity": 40},
                "unit": "ops"
            },
            "overrides": []
        },
        "gridPos": {"h": 6, "w": 10, "x": 14, "y": 29},
        "id": pid,
        "options": {"legend": {"displayMode": "list", "placement": "bottom"}},
        "targets": [{
            "datasource": {"type": "loki", "uid": "loki"},
            "editorMode": "code",
            "expr": 'sum by (container) (rate({job="docker"} |= "error" [5m]))',
            "legendFormat": "{{container}} errors",
            "refId": "A"
        }],
        "title": "Application Error Log Spike Rate",
        "type": "timeseries"
    })
    pid += 1

    data["panels"] = panels
    data["title"] = "ATLSD Production Command Center"
    data["version"] = data.get("version", 1) + 1

    with open('infra/monitoring/grafana/dashboards/atlsd-production-overview.json', 'w') as f:
        json.dump(data, f, indent=2)

    print(f"Successfully generated clean dashboard with {len(panels)} panels!")

if __name__ == '__main__':
    build_dashboard()
