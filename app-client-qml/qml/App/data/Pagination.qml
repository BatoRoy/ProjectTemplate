pragma ComponentBehavior: Bound

// Pagination — the twin of components/data/Pagination.tsx: page buttons with ellipses,
// so a hundred pages do not become a hundred buttons.

import QtQuick
import QtQuick.Layouts
import App

RowLayout {
    id: root

    property int page: 1
    property int pages: 1
    // How many pages to show either side of the current one before eliding.
    property int window: 1
    signal picked(int page)

    spacing: Theme.px(4)

    // Without a zero minimum the enclosing layout cannot squeeze this row, so `width`
    // stays at the full implicit width and `compact` below can never become true — the
    // row would be too wide *because* it had not collapsed, and not collapse *because*
    // it was too wide.
    Layout.minimumWidth: 0

    // Below this, the numbered slots are dropped for a compact readout. A pager with
    // twenty pages is wider than a narrow card, and a row of fixed-width buttons has no
    // way to shrink — so it would push its last button outside the card instead.
    readonly property int slotsWidth: (slots.length + 2) * (Theme.controlHeight + Theme.px(4))
    readonly property bool compact: width > 0 && width < slotsWidth

    // [1, "…", 4, 5, 6, "…", 20]
    readonly property var slots: {
        var out = [], last = 0;
        for (var i = 1; i <= pages; ++i) {
            var near = Math.abs(i - page) <= window;
            if (i === 1 || i === pages || near) {
                if (last && i - last > 1)
                    out.push("…");
                out.push(i);
                last = i;
            }
        }
        return out;
    }

    function go(p) {
        var next = Math.max(1, Math.min(root.pages, p));
        if (next !== root.page) {
            root.page = next;
            root.picked(next);
        }
    }

    Button {
        variant: "ghost"
        icon: "chevron-left"
        text: ""
        enabled: root.page > 1
        onClicked: root.go(root.page - 1)
    }

    Txt {
        visible: root.compact
        Layout.fillWidth: true
        Layout.minimumWidth: 0
        text: root.page + " / " + root.pages
        pixelSize: Theme.fontXs
        color: Theme.subtext
        horizontalAlignment: Text.AlignHCenter
    }

    Repeater {
        model: root.compact ? [] : root.slots
        delegate: Item {
            id: slot
            required property var modelData
            readonly property bool isGap: modelData === "…"

            implicitWidth: Theme.controlHeight
            implicitHeight: Theme.controlHeight

            Rectangle {
                anchors.fill: parent
                visible: !slot.isGap
                radius: Theme.radius
                color: slot.modelData === root.page ? Theme.accentTintMed : slotMouse.containsMouse ? Theme.alpha(Theme.card, 0.6) : "transparent"
                border.width: 1
                border.color: slot.modelData === root.page ? Theme.accentRing : "transparent"
            }

            Txt {
                anchors.centerIn: parent
                text: slot.modelData
                pixelSize: Theme.fontXs
                weight: slot.modelData === root.page ? Font.Medium : Font.Normal
                color: slot.isGap ? Theme.muted : slot.modelData === root.page ? Theme.accentText : Theme.subtext
            }

            MouseArea {
                id: slotMouse
                anchors.fill: parent
                enabled: !slot.isGap
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: root.go(slot.modelData)
            }
        }
    }

    Button {
        variant: "ghost"
        icon: "chevron-right"
        text: ""
        enabled: root.page < root.pages
        onClicked: root.go(root.page + 1)
    }
}
