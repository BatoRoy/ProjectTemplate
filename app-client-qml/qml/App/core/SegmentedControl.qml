// SegmentedControl — the twin of components/layout/SegmentedControl.tsx.
//
// A track with a sliding active pill. The pill is one Rectangle that animates between
// positions rather than a per-segment background, so the movement reads as one control
// changing state instead of two independent fades.

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts
import App

Rectangle {
    id: root

    // [{ value, label }]
    property var options: []
    property var value: undefined
    property bool small: false
    signal picked(var value)

    readonly property int segHeight: small ? Theme.px(26) : Theme.controlHeight
    readonly property int index: {
        for (var i = 0; i < options.length; ++i)
            if (options[i].value === value)
                return i;
        return -1;
    }

    color: Theme.composite(Theme.border, Theme.card, 0.5)
    radius: Theme.radius
    implicitHeight: segHeight + Theme.px(6)
    implicitWidth: row.implicitWidth + Theme.px(6)

    Rectangle {
        id: pill
        visible: root.index >= 0
        y: Theme.px(3)
        height: root.segHeight
        width: root.options.length > 0 ? (root.width - Theme.px(6)) / root.options.length : 0
        x: Theme.px(3) + width * Math.max(0, root.index)
        radius: Theme.radiusSm
        color: Theme.accentTintMed
        border.width: 1
        border.color: Theme.accentRing
        Behavior on x {
            NumberAnimation {
                duration: Theme.animFast
                easing.type: Easing.OutCubic
            }
        }
    }

    RowLayout {
        id: row
        anchors.fill: parent
        anchors.margins: Theme.px(3)
        spacing: 0

        Repeater {
            model: root.options
            delegate: Item {
                id: seg
                required property var modelData
                required property int index

                // Both are needed. fillWidth equalises the segments when there is room;
                // implicitWidth is what the *control* reports upward, and without it every
                // segment claims zero, the whole control claims ~nothing, and a parent
                // sizing it to its content clips the labels.
                implicitWidth: segLabel.implicitWidth + Theme.space4
                Layout.fillWidth: true
                Layout.minimumWidth: 0
                Layout.preferredHeight: root.segHeight

                Txt {
                    id: segLabel
                    anchors.centerIn: parent
                    text: seg.modelData.label
                    pixelSize: Theme.fontXs
                    weight: Font.Medium
                    color: seg.index === root.index ? Theme.accentText : segMouse.containsMouse ? Theme.text : Theme.muted
                    elide: Text.ElideRight
                    // Guarded against seg.width being 0 on the first pass: the naive
                    // Math.min(implicitWidth, seg.width - space2) evaluates negative there,
                    // an elided Text given a negative width reports ~0 implicit width, and
                    // the segment then reports that upward — so the whole control claimed
                    // 38px and clipped its own labels.
                    width: seg.width > Theme.space2 ? Math.min(implicitWidth, seg.width - Theme.space2) : implicitWidth
                }

                MouseArea {
                    id: segMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: {
                        root.value = seg.modelData.value;
                        root.picked(seg.modelData.value);
                    }
                }
            }
        }
    }
}
