pragma ComponentBehavior: Bound

// BarChart — the twin of the BarChart in components/data/Charts.tsx.
//
// `points` is an array of numbers or of { value, label }.

import QtQuick
import QtQuick.Layouts
import App

Item {
    id: root

    property var points: []
    property color fill: Theme.accent
    property bool showLabels: true

    implicitHeight: Theme.px(140)

    readonly property var values: points.map(p => (typeof p === "number" ? p : p.value))
    readonly property real hi: {
        var m = Math.max.apply(null, values.concat([0]));
        return m === 0 ? 1 : m;
    }
    readonly property int labelRow: showLabels ? Theme.px(16) : 0

    Row {
        anchors.fill: parent
        spacing: Theme.px(4)

        Repeater {
            model: root.points
            delegate: Item {
                id: bar
                required property var modelData
                required property int index
                readonly property real value: typeof modelData === "number" ? modelData : modelData.value

                width: (root.width - Theme.px(4) * (root.points.length - 1)) / Math.max(1, root.points.length)
                height: root.height

                Rectangle {
                    anchors.bottom: parent.bottom
                    anchors.bottomMargin: root.labelRow
                    width: parent.width
                    // Floor at 2px so a zero value is still visibly a bar rather than
                    // nothing at all.
                    height: Math.max(2, (root.height - root.labelRow) * bar.value / root.hi)
                    radius: Theme.radiusSm
                    color: barMouse.containsMouse ? Theme.accentHover : root.fill
                    Behavior on color {
                        ColorAnimation {
                            duration: Theme.animFast
                        }
                    }
                    Behavior on height {
                        NumberAnimation {
                            duration: Theme.animBase
                            easing.type: Easing.OutCubic
                        }
                    }
                }

                Txt {
                    anchors.bottom: parent.bottom
                    anchors.horizontalCenter: parent.horizontalCenter
                    visible: root.showLabels
                    width: parent.width
                    horizontalAlignment: Text.AlignHCenter
                    text: typeof bar.modelData === "number" ? "" : (bar.modelData.label ?? "")
                    pixelSize: Theme.fontXs
                    color: Theme.muted
                    elide: Text.ElideRight
                }

                MouseArea {
                    id: barMouse
                    anchors.fill: parent
                    hoverEnabled: true
                }
            }
        }
    }
}
