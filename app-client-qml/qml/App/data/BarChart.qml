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

    property int hoverIndex: -1

    // Readout for the hovered bar. Declared after the Row so it draws above the bars, and
    // clamped to the chart so a bar at either end does not push it outside the card.
    Rectangle {
        id: tip
        z: 1
        visible: root.hoverIndex >= 0
        width: tipText.implicitWidth + Theme.space2 * 2
        height: tipText.implicitHeight + Theme.space1 * 2
        radius: Theme.radiusSm
        color: Theme.isLight ? Theme.text : Theme.surface
        border.width: 1
        border.color: Theme.border

        readonly property real barWidth: root.points.length > 0 ? (root.width - Theme.px(4) * (root.points.length - 1)) / root.points.length : 0
        readonly property real barValue: root.hoverIndex >= 0 ? root.values[root.hoverIndex] : 0

        x: Math.max(0, Math.min(root.width - width, root.hoverIndex * (barWidth + Theme.px(4)) + barWidth / 2 - width / 2))
        y: Math.max(0, root.height - root.labelRow - Math.max(2, (root.height - root.labelRow) * barValue / root.hi) - height - Theme.space1)

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
                    acceptedButtons: Qt.NoButton
                    onContainsMouseChanged: root.hoverIndex = containsMouse ? bar.index : -1
                }
            }
        }
    }
}
