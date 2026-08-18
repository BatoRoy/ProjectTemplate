// Spinner is the twin of Spinner in components/Feedback.tsx — an accent arc that
// rotates. Drawn with Shape rather than an animated image so it stays crisp at any
// UI scale and takes its colour from a property.

import QtQuick
import QtQuick.Shapes
import App

Item {
    id: root

    property int size: Theme.iconSize
    property color color: Theme.accent
    property int thickness: Math.max(2, Math.round(size / 8))

    implicitWidth: size
    implicitHeight: size

    Shape {
        id: shape
        anchors.fill: parent
        // Antialiasing is off by default on Shape and the arc looks stepped without it.
        preferredRendererType: Shape.CurveRenderer

        ShapePath {
            strokeColor: root.color
            strokeWidth: root.thickness
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap

            PathAngleArc {
                centerX: root.size / 2
                centerY: root.size / 2
                radiusX: (root.size - root.thickness) / 2
                radiusY: (root.size - root.thickness) / 2
                startAngle: -90
                // Three quarters, so there is a visible gap that reads as motion.
                sweepAngle: 270
            }
        }
    }

    RotationAnimation {
        target: shape
        running: root.visible
        loops: Animation.Infinite
        from: 0
        to: 360
        duration: 900
    }
}
