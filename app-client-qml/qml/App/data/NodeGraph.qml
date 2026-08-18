pragma ComponentBehavior: Bound

// NodeGraph — the twin of components/data/NodeGraph.tsx.
//
// The React one wraps @xyflow/react and mostly supplies a themed node. QML has no
// equivalent library, so this is the whole thing from scratch: pan, zoom, node dragging,
// bezier edges, and a themed node matching the web flavor's.
//
// The parts worth knowing about, because they are what a from-scratch graph gets wrong:
//
//   Zoom is anchored to the cursor, not the origin. Zooming to the top-left is the
//   giveaway that a canvas was written quickly — the thing under the pointer has to stay
//   under the pointer, which means the pan offset moves with the scale.
//
//   Edges are drawn on a Canvas, not with Shape/ShapePath. Two reasons: a Repeater cannot
//   produce ShapePath delegates at all (they are not Items, and it refuses with "Delegate
//   must be of Item type"), and one Shape per edge is a scene-graph node per edge, which a
//   graph of any size spends more time assembling than drawing. One canvas pass draws the
//   grid and every edge together.
//
//   Node positions live in `nodes`, not in the delegates. A delegate that owns its own x/y
//   loses it the moment the view re-creates it, and the caller can never read the layout
//   back out.
//
// nodes: [{ id, x, y, label, sublabel?, tone? }]
// edges: [{ from, to }]

import QtQuick
import App

Rectangle {
    id: root

    property var nodes: []
    property var edges: []
    property real minZoom: 0.4
    property real maxZoom: 2.5
    property bool showControls: true
    signal nodeClicked(var node)
    signal nodeMoved(string id, real x, real y)

    implicitHeight: Theme.px(320)
    color: Theme.bg
    border.color: Theme.border
    border.width: 1
    radius: Theme.radiusXl
    clip: true

    property real zoom: 1.0
    property real panX: 0
    property real panY: 0

    readonly property int nodeWidth: Theme.px(140)
    readonly property int nodeHeight: Theme.px(52)

    function nodeById(id) {
        for (var i = 0; i < nodes.length; ++i)
            if (nodes[i].id === id)
                return nodes[i];
        return null;
    }

    function toneColor(n) {
        switch (n && n.tone) {
        case "success":
            return Theme.green;
        case "warning":
            return Theme.yellow;
        case "error":
            return Theme.red;
        default:
            return Theme.accent;
        }
    }

    // Move a node by writing back into the model, so the caller owns the layout.
    function moveNode(id, x, y) {
        var next = nodes.slice();
        for (var i = 0; i < next.length; ++i) {
            if (next[i].id === id) {
                next[i] = Object.assign({}, next[i], {
                    x: x,
                    y: y
                });
                break;
            }
        }
        nodes = next;
        root.nodeMoved(id, x, y);
    }

    function fit() {
        if (nodes.length === 0)
            return;
        var minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
        for (var i = 0; i < nodes.length; ++i) {
            minX = Math.min(minX, nodes[i].x);
            minY = Math.min(minY, nodes[i].y);
            maxX = Math.max(maxX, nodes[i].x + nodeWidth);
            maxY = Math.max(maxY, nodes[i].y + nodeHeight);
        }
        var pad = Theme.space5;
        var z = Math.min((width - pad * 2) / (maxX - minX), (height - pad * 2) / (maxY - minY));
        zoom = Math.max(minZoom, Math.min(maxZoom, z));
        panX = (width - (maxX - minX) * zoom) / 2 - minX * zoom;
        panY = (height - (maxY - minY) * zoom) / 2 - minY * zoom;
    }

    // Dotted background, matching the web flavor's. Drawn in screen space and offset by
    // the pan so it reads as the canvas moving rather than the nodes sliding over paper.
    Canvas {
        anchors.fill: parent
        renderStrategy: Canvas.Cooperative

        Connections {
            target: root
            function onPanXChanged() {
                parent.requestPaint();
            }
            function onPanYChanged() {
                parent.requestPaint();
            }
            function onZoomChanged() {
                parent.requestPaint();
            }
        }

        Connections {
            target: root
            function onNodesChanged() {
                parent.requestPaint();
            }
            function onEdgesChanged() {
                parent.requestPaint();
            }
        }

        // Graph coordinates → screen coordinates. Everything below draws in screen space,
        // because a Canvas has no transform of its own the way the node layer does.
        function sx(gx) {
            return root.panX + gx * root.zoom;
        }
        function sy(gy) {
            return root.panY + gy * root.zoom;
        }

        onPaint: {
            var ctx = getContext("2d");
            ctx.reset();

            // Dots
            var gap = Theme.px(18) * root.zoom;
            if (gap >= 6) {
                ctx.fillStyle = Theme.alpha(Theme.border, 0.9);
                var ox = root.panX % gap, oy = root.panY % gap;
                for (var x = ox; x < width; x += gap)
                    for (var y = oy; y < height; y += gap)
                        ctx.fillRect(x, y, 1.5, 1.5);
            }

            // Edges
            ctx.strokeStyle = Theme.alpha(Theme.subtext, 0.7);
            ctx.lineWidth = 1.5 * root.zoom;
            ctx.lineCap = "round";
            for (var i = 0; i < root.edges.length; ++i) {
                var a = root.nodeById(root.edges[i].from);
                var b = root.nodeById(root.edges[i].to);
                if (!a || !b)
                    continue;

                var x1 = sx(a.x + root.nodeWidth), y1 = sy(a.y + root.nodeHeight / 2);
                var x2 = sx(b.x), y2 = sy(b.y + root.nodeHeight / 2);
                // Horizontal control points, so edges leave and enter sideways — the shape
                // that makes a left-to-right flow readable.
                var dx = Math.max(Theme.px(40) * root.zoom, Math.abs(x2 - x1) / 2);

                ctx.beginPath();
                ctx.moveTo(x1, y1);
                ctx.bezierCurveTo(x1 + dx, y1, x2 - dx, y2, x2, y2);
                ctx.stroke();
            }
        }
    }

    // The canvas: one item that pans and scales, with everything inside it in graph
    // coordinates. Nodes therefore never need to know about the viewport.
    Item {
        id: canvas
        x: root.panX
        y: root.panY
        transform: Scale {
            origin.x: 0
            origin.y: 0
            xScale: root.zoom
            yScale: root.zoom
        }

        Repeater {
            model: root.nodes
            delegate: Rectangle {
                id: node
                required property var modelData

                x: modelData.x
                y: modelData.y
                width: root.nodeWidth
                height: root.nodeHeight
                radius: Theme.radius
                color: Theme.card
                border.width: 1
                border.color: nodeMouse.drag.active ? Theme.accentRing : nodeMouse.containsMouse ? Theme.alpha(root.toneColor(modelData), 0.6) : Theme.border

                // A tone stripe rather than a tinted fill, so a graph of coloured nodes
                // does not become unreadable.
                Rectangle {
                    width: Theme.px(3)
                    height: parent.height
                    radius: width / 2
                    color: root.toneColor(node.modelData)
                }

                Txt {
                    x: Theme.space3
                    y: Theme.space2
                    width: parent.width - Theme.space3 - Theme.space2
                    text: node.modelData.label ?? ""
                    pixelSize: Theme.fontSm
                    weight: Font.Medium
                    color: Theme.text
                    elide: Text.ElideRight
                }
                Txt {
                    x: Theme.space3
                    y: Theme.space2 + Theme.px(16)
                    width: parent.width - Theme.space3 - Theme.space2
                    visible: node.modelData.sublabel !== undefined
                    text: node.modelData.sublabel ?? ""
                    pixelSize: Theme.fontXs
                    color: Theme.muted
                    elide: Text.ElideRight
                }

                MouseArea {
                    id: nodeMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: drag.active ? Qt.ClosedHandCursor : Qt.OpenHandCursor
                    drag.target: node

                    // Write the position back to the model on release rather than per
                    // frame: rebuilding the nodes array on every mouse move would rebuild
                    // every delegate with it.
                    onReleased: root.moveNode(node.modelData.id, node.x, node.y)
                    onClicked: root.nodeClicked(node.modelData)
                }
            }
        }
    }

    // Panning, and zooming anchored to the cursor.
    MouseArea {
        anchors.fill: parent
        // Below the nodes, so dragging a node does not also pan the canvas.
        z: -1
        acceptedButtons: Qt.LeftButton | Qt.MiddleButton
        cursorShape: pressed ? Qt.ClosedHandCursor : Qt.ArrowCursor

        property real lastX: 0
        property real lastY: 0
        onPressed: e => {
            lastX = e.x;
            lastY = e.y;
        }
        onPositionChanged: e => {
            if (!pressed)
                return;
            root.panX += e.x - lastX;
            root.panY += e.y - lastY;
            lastX = e.x;
            lastY = e.y;
        }

        onWheel: wheel => {
            var factor = wheel.angleDelta.y > 0 ? 1.12 : 1 / 1.12;
            var next = Math.max(root.minZoom, Math.min(root.maxZoom, root.zoom * factor));
            if (next === root.zoom)
                return;
            // Keep the graph point under the cursor under the cursor: solve for the pan
            // that leaves (wheel.x, wheel.y) mapping to the same graph coordinate.
            var gx = (wheel.x - root.panX) / root.zoom;
            var gy = (wheel.y - root.panY) / root.zoom;
            root.zoom = next;
            root.panX = wheel.x - gx * next;
            root.panY = wheel.y - gy * next;
        }
    }

    // Controls
    Row {
        visible: root.showControls
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: Theme.space3
        spacing: Theme.space1

        Button {
            variant: "ghost"
            icon: "minus"
            text: ""
            onClicked: root.zoom = Math.max(root.minZoom, root.zoom / 1.2)
        }
        Button {
            variant: "ghost"
            icon: "plus"
            text: ""
            onClicked: root.zoom = Math.min(root.maxZoom, root.zoom * 1.2)
        }
        Button {
            variant: "ghost"
            text: qsTr("Fit")
            onClicked: root.fit()
        }
    }

    MonoText {
        visible: root.showControls
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        anchors.margins: Theme.space3
        text: Math.round(root.zoom * 100) + "%"
        color: Theme.muted
    }
}
