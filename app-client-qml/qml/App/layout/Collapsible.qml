// Collapsible — one disclosure row. The twin of the Collapsible exported alongside
// Accordion in components/layout/Accordion.tsx.
//
// The body animates its height rather than its visibility, because a section that pops
// open reads as a layout jump instead of a disclosure. clip is on so the text does not
// spill during the animation.

import QtQuick
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property string title: ""
    property string text: ""
    property bool expanded: false
    signal toggled

    color: Theme.card
    border.color: Theme.border
    border.width: 1
    radius: Theme.radius
    clip: true
    // The header is anchored with space3 margins, so its own implicitHeight does not
    // include them. Leaving them out clipped the body by exactly two margins — visible
    // only once a section was actually expanded.
    implicitHeight: header.implicitHeight + Theme.space3 * 2 + (expanded ? body.implicitHeight + Theme.space3 : 0)
    Behavior on implicitHeight {
        NumberAnimation {
            duration: Theme.animBase
            easing.type: Easing.OutCubic
        }
    }

    RowLayout {
        id: header
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.margins: Theme.space3
        spacing: Theme.space2

        Txt {
            Layout.fillWidth: true
            Layout.minimumWidth: 0
            text: root.title
            pixelSize: Theme.fontSm
            weight: Font.Medium
            color: Theme.text
            elide: Text.ElideRight
        }

        Icon {
            name: "chevron-down"
            size: Theme.px(14)
            color: Theme.muted
            rotation: root.expanded ? 180 : 0
            Behavior on rotation {
                NumberAnimation {
                    duration: Theme.animFast
                }
            }
        }
    }

    Txt {
        id: body
        anchors.top: header.bottom
        anchors.topMargin: Theme.space2
        anchors.left: parent.left
        anchors.leftMargin: Theme.space3
        anchors.right: parent.right
        anchors.rightMargin: Theme.space3
        text: root.text
        pixelSize: Theme.fontXs
        color: Theme.subtext
        wrapMode: Text.WordWrap
        opacity: root.expanded ? 1 : 0
        // visible follows opacity so a collapsed body is not merely transparent: an
        // invisible-but-laid-out child still occupies its parent's box, which reads as an
        // overflow to anything measuring geometry (and to a screen reader as present).
        visible: opacity > 0
        Behavior on opacity {
            NumberAnimation {
                duration: Theme.animFast
            }
        }
    }

    MouseArea {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: header.implicitHeight + Theme.space3 * 2
        cursorShape: Qt.PointingHandCursor
        onClicked: {
            root.expanded = !root.expanded;
            root.toggled();
        }
    }
}
