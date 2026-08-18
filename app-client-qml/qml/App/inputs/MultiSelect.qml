pragma ComponentBehavior: Bound

// MultiSelect — the twin of components/inputs/MultiSelect.tsx: a searchable multi-choice
// whose trigger shows the chosen values as chips.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Item {
    id: root

    // [{ value, label }]
    property var options: []
    property var values: []
    property string placeholder: qsTr("Choose…")
    signal changed(var values)

    implicitHeight: Math.max(Theme.controlHeight, chips.implicitHeight + Theme.space2 * 2)
    implicitWidth: Theme.px(240)

    function toggle(v) {
        var next = root.values.slice();
        var i = next.indexOf(v);
        if (i === -1)
            next.push(v);
        else
            next.splice(i, 1);
        root.values = next;
        root.changed(next);
    }
    function labelFor(v) {
        for (var i = 0; i < options.length; ++i)
            if (options[i].value === v)
                return options[i].label;
        return v;
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.isLight ? Theme.surface : Theme.bg
        border.width: 1
        border.color: popup.opened ? Theme.accentRing : triggerMouse.containsMouse ? Theme.accentRing : Theme.border
        radius: Theme.radius
        Behavior on border.color {
            ColorAnimation {
                duration: Theme.animFast
            }
        }

        Flow {
            id: chips
            anchors.fill: parent
            anchors.margins: Theme.space2
            spacing: Theme.space1

            Txt {
                visible: root.values.length === 0
                text: root.placeholder
                color: Theme.muted
                pixelSize: Theme.fontSm
                height: Theme.px(22)
            }

            Repeater {
                model: root.values
                delegate: Rectangle {
                    id: chip
                    required property var modelData
                    height: Theme.px(22)
                    width: chipLabel.implicitWidth + Theme.space3
                    radius: Theme.radiusFull
                    color: Theme.accentTintMed

                    Txt {
                        id: chipLabel
                        anchors.centerIn: parent
                        text: root.labelFor(chip.modelData)
                        pixelSize: Theme.fontXs
                        color: Theme.accentText
                    }
                }
            }
        }

        MouseArea {
            id: triggerMouse
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: popup.opened ? popup.close() : popup.open()
        }
    }

    Popover {
        id: popup
        padding: 0
        width: root.width

        // MenuList already renders a checkmark per row, so a multi-select is the same
        // list with `checked` derived from membership rather than equality.
        MenuList {
            width: parent.width
            items: root.options.map(o => ({
                        label: o.label,
                        checked: root.values.indexOf(o.value) !== -1,
                        value: o.value
                    }))
            // Stays open: choosing several things one popup at a time is the wrong shape.
            onTriggered: (index, item) => root.toggle(item.value)
        }
    }
}
