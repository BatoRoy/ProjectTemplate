// LineChart — the twin of the LineChart in components/data/Charts.tsx.
//
// Hand-rolled with QtQuick.Shapes rather than QtCharts, and that is a deliberate
// dependency decision: QtCharts is a separate package (not installed on the reference
// machine, and GPL/commercial rather than LGPL), so requiring it would add a `requires`
// entry and an install step to every app that draws one line. Shapes ships with
// qt6-declarative, which the app already depends on.
//
// `points` is a plain array of numbers, or of { value, label } objects.

import QtQuick
import QtQuick.Shapes
import App

Item {
    id: root

    property var points: []
    property color stroke: Theme.accent
    property bool area: false
    property bool grid: true
    property real minY: NaN     // auto
    property real maxY: NaN
    property int lines: 4       // horizontal grid lines

    implicitHeight: Theme.px(140)

    readonly property var values: points.map(p => (typeof p === "number" ? p : p.value))
    readonly property real lo: isNaN(minY) ? Math.min.apply(null, values.concat([0])) : minY
    readonly property real hi: {
        if (!isNaN(maxY))
            return maxY;
        var m = Math.max.apply(null, values.concat([0]));
        // A flat series would divide by zero; give it a nominal range instead.
        return m === lo ? lo + 1 : m;
    }

    function xAt(i) {
        return values.length < 2 ? 0 : width * i / (values.length - 1);
    }
    function yAt(v) {
        return height - (height * (v - lo) / (hi - lo));
    }

    // Grid, drawn under the series.
    Repeater {
        model: root.grid ? root.lines + 1 : 0
        delegate: Rectangle {
            required property int index
            width: root.width
            height: 1
            y: root.height * index / root.lines
            color: Theme.alpha(Theme.border, 0.6)
        }
    }

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        visible: root.values.length > 1

        // Fill under the line. A separate path from the stroke so the stroke does not
        // trace the baseline back to the start.
        ShapePath {
            fillColor: root.area ? Theme.alpha(root.stroke, 0.15) : "transparent"
            strokeColor: "transparent"
            PathPolyline {
                path: {
                    if (!root.area || root.values.length < 2)
                        return [];
                    var pts = [Qt.point(root.xAt(0), root.height)];
                    for (var i = 0; i < root.values.length; ++i)
                        pts.push(Qt.point(root.xAt(i), root.yAt(root.values[i])));
                    pts.push(Qt.point(root.xAt(root.values.length - 1), root.height));
                    return pts;
                }
            }
        }

        ShapePath {
            strokeColor: root.stroke
            strokeWidth: 2
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathPolyline {
                path: {
                    var pts = [];
                    for (var i = 0; i < root.values.length; ++i)
                        pts.push(Qt.point(root.xAt(i), root.yAt(root.values[i])));
                    return pts;
                }
            }
        }
    }

    // ── Hover readout ────────────────────────────────────────────────────────
    // A line has no per-item hit target the way a bar chart does, so the nearest point to
    // the cursor is found by index: the x spacing is uniform, so it is a divide rather
    // than a search.
    property int hoverIndex: -1

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.NoButton
        onExited: root.hoverIndex = -1
        onPositionChanged: e => {
            if (root.values.length < 2) {
                root.hoverIndex = -1;
                return;
            }
            var stepX = root.width / (root.values.length - 1);
            root.hoverIndex = Math.max(0, Math.min(root.values.length - 1, Math.round(e.x / stepX)));
        }
    }

    // Guide line + point marker
    Rectangle {
        visible: root.hoverIndex >= 0
        x: root.hoverIndex >= 0 ? root.xAt(root.hoverIndex) : 0
        width: 1
        height: root.height
        color: Theme.alpha(Theme.subtext, 0.5)
    }

    Rectangle {
        visible: root.hoverIndex >= 0
        width: Theme.px(8)
        height: width
        radius: width / 2
        x: (root.hoverIndex >= 0 ? root.xAt(root.hoverIndex) : 0) - width / 2
        y: (root.hoverIndex >= 0 ? root.yAt(root.values[root.hoverIndex]) : 0) - height / 2
        color: root.stroke
        border.width: 2
        border.color: Theme.card
    }

    Rectangle {
        id: tip
        visible: root.hoverIndex >= 0
        // Clamped to the chart, so a point near either edge does not push the readout
        // outside the card.
        x: Math.max(0, Math.min(root.width - width, (root.hoverIndex >= 0 ? root.xAt(root.hoverIndex) : 0) - width / 2))
        y: Math.max(0, (root.hoverIndex >= 0 ? root.yAt(root.values[root.hoverIndex]) : 0) - height - Theme.space2)
        width: tipText.implicitWidth + Theme.space2 * 2
        height: tipText.implicitHeight + Theme.space1 * 2
        radius: Theme.radiusSm
        color: Theme.isLight ? Theme.text : Theme.surface
        border.width: 1
        border.color: Theme.border

        Txt {
            id: tipText
            anchors.centerIn: parent
            text: {
                if (root.hoverIndex < 0)
                    return "";
                var p = root.points[root.hoverIndex];
                var label = (typeof p === "object" && p.label !== undefined) ? p.label + "  " : "";
                return label + root.values[root.hoverIndex];
            }
            pixelSize: Theme.fontXs
            family: Theme.fontMono
            color: Theme.isLight ? Theme.bg : Theme.text
        }
    }

    EmptyState {
        anchors.centerIn: parent
        visible: root.values.length < 2
        icon: "activity"
        title: qsTr("Not enough data")
    }
}
