// CircularProgress — the twin of CircularProgress in components/layout/Progress.tsx.
//
// Drawn with Shape/PathAngleArc rather than an SVG so the sweep is a property rather
// than a redraw, and so it stays crisp at any UI scale.

import QtQuick
import QtQuick.Shapes
import App

Item {
    id: root

    property real value: 0        // 0–100
    property int size: Theme.px(48)
    property int thickness: Theme.px(4)
    property bool showLabel: false

    implicitWidth: size
    implicitHeight: size

    readonly property real clamped: Math.max(0, Math.min(100, value))

    Shape {
        anchors.fill: parent
        preferredRendererType: Shape.CurveRenderer

        // Track
        ShapePath {
            strokeColor: Theme.composite(Theme.border, Theme.card, 0.8)
            strokeWidth: root.thickness
            fillColor: "transparent"
            PathAngleArc {
                centerX: root.size / 2
                centerY: root.size / 2
                radiusX: (root.size - root.thickness) / 2
                radiusY: (root.size - root.thickness) / 2
                startAngle: 0
                sweepAngle: 360
            }
        }

        // Value, starting at 12 o'clock like the web flavor's rotated SVG.
        ShapePath {
            strokeColor: Theme.accent
            strokeWidth: root.thickness
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            PathAngleArc {
                centerX: root.size / 2
                centerY: root.size / 2
                radiusX: (root.size - root.thickness) / 2
                radiusY: (root.size - root.thickness) / 2
                startAngle: -90
                sweepAngle: 360 * root.clamped / 100
                Behavior on sweepAngle {
                    NumberAnimation {
                        duration: Theme.animBase
                    }
                }
            }
        }
    }

    Txt {
        anchors.centerIn: parent
        visible: root.showLabel
        text: Math.round(root.clamped) + "%"
        pixelSize: Theme.fontXs
        weight: Font.Medium
        color: Theme.text
    }
}
