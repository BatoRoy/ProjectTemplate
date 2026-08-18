// Sparkline — the twin of the Sparkline in components/data/Charts.tsx: a bare trend line
// for sitting inline in a row or a stat tile. No axes, no grid, no empty state.

import QtQuick
import QtQuick.Shapes
import App

Item {
    id: root

    property var points: []
    property color stroke: Theme.accent

    implicitWidth: Theme.px(80)
    implicitHeight: Theme.px(20)

    readonly property var values: points.map(p => (typeof p === "number" ? p : p.value))
    readonly property real lo: Math.min.apply(null, values.concat([0]))
    readonly property real hi: {
        var m = Math.max.apply(null, values.concat([0]));
        return m === lo ? lo + 1 : m;
    }

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer
        visible: root.values.length > 1

        ShapePath {
            strokeColor: root.stroke
            strokeWidth: 1.5
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            PathPolyline {
                path: {
                    var pts = [];
                    for (var i = 0; i < root.values.length; ++i) {
                        var x = root.width * i / (root.values.length - 1);
                        var y = root.height - (root.height * (root.values[i] - root.lo) / (root.hi - root.lo));
                        pts.push(Qt.point(x, y));
                    }
                    return pts;
                }
            }
        }
    }
}
