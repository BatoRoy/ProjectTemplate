pragma ComponentBehavior: Bound

// DataTable — the twin of components/data/DataTable.tsx: a sortable table with a sticky
// header and hover rows.
//
// columns: [{ key, label, width?, align?, mono? }]
// rows:    [{ <key>: value, … }]
//
// Sorting is done here rather than expected from the caller, matching the React version:
// a table that cannot sort itself pushes the same twenty lines into every page that uses
// one. Pass `sortable: false` for server-side ordering.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property var columns: []
    property var rows: []
    property bool sortable: true
    property string sortKey: ""
    property bool sortDescending: false
    property string emptyText: qsTr("No rows")
    signal rowClicked(int index, var row)

    color: Theme.card
    border.color: Theme.border
    border.width: 1
    radius: Theme.radiusXl
    clip: true
    implicitHeight: Theme.px(240)

    readonly property var sorted: {
        if (!sortKey || !sortable)
            return rows;
        var copy = rows.slice();
        copy.sort(function (a, b) {
            var x = a[root.sortKey], y = b[root.sortKey];
            if (x === y)
                return 0;
            // Numbers compare numerically, everything else as a localised string —
            // otherwise "10" sorts before "9".
            var r = (typeof x === "number" && typeof y === "number") ? x - y : String(x).localeCompare(String(y));
            return root.sortDescending ? -r : r;
        });
        return copy;
    }

    function widthFor(col, total) {
        // An explicit width is a share, not pixels: the columns always fill the table.
        var totalShares = 0;
        for (var i = 0; i < root.columns.length; ++i)
            totalShares += root.columns[i].width ?? 1;
        return total * (col.width ?? 1) / totalShares;
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Header — outside the ListView, which is what makes it sticky without a
        // per-frame position update.
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: Theme.px(36)
            color: Theme.composite(Theme.border, Theme.card, 0.35)

            Row {
                anchors.fill: parent
                anchors.leftMargin: Theme.space3
                anchors.rightMargin: Theme.space3

                Repeater {
                    model: root.columns
                    delegate: Item {
                        id: head
                        required property var modelData
                        width: root.widthFor(modelData, parent.width)
                        height: parent.height

                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: head.modelData.align === "right" ? undefined : parent.left
                            anchors.right: head.modelData.align === "right" ? parent.right : undefined
                            anchors.rightMargin: head.modelData.align === "right" ? Theme.space2 : 0
                            spacing: Theme.space1

                            Txt {
                                anchors.verticalCenter: parent.verticalCenter
                                text: head.modelData.label ?? head.modelData.key
                                pixelSize: Theme.fontXs
                                weight: Font.DemiBold
                                color: root.sortKey === head.modelData.key ? Theme.accentText : Theme.subtext
                                capitalization: Font.AllUppercase
                                letterSpacing: 0.6
                            }

                            Icon {
                                anchors.verticalCenter: parent.verticalCenter
                                visible: root.sortable && root.sortKey === head.modelData.key
                                name: root.sortDescending ? "chevron-down" : "chevron-up"
                                size: Theme.px(11)
                                color: Theme.accentText
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            enabled: root.sortable
                            cursorShape: Qt.PointingHandCursor
                            onClicked: {
                                if (root.sortKey === head.modelData.key)
                                    root.sortDescending = !root.sortDescending;
                                else {
                                    root.sortKey = head.modelData.key;
                                    root.sortDescending = false;
                                }
                            }
                        }
                    }
                }
            }

            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: Theme.border
            }
        }

        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: root.sorted
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {}

            delegate: Rectangle {
                id: row
                required property var modelData
                required property int index

                width: list.width
                height: Theme.rowHeight
                color: rowMouse.containsMouse ? Theme.alpha(Theme.border, 0.4) : "transparent"

                Row {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space3
                    anchors.rightMargin: Theme.space3

                    Repeater {
                        model: root.columns
                        delegate: Item {
                            id: cell
                            required property var modelData
                            width: root.widthFor(modelData, parent.width)
                            height: parent.height

                            Txt {
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: cell.modelData.align === "right" ? undefined : parent.left
                                anchors.right: cell.modelData.align === "right" ? parent.right : undefined
                                anchors.rightMargin: cell.modelData.align === "right" ? Theme.space2 : 0
                                width: Math.min(implicitWidth, cell.width - Theme.space2)
                                text: String(row.modelData[cell.modelData.key] ?? "")
                                pixelSize: Theme.fontSm
                                family: cell.modelData.mono === true ? Theme.fontMono : Theme.fontSans
                                color: Theme.text
                                elide: Text.ElideRight
                            }
                        }
                    }
                }

                Rectangle {
                    anchors.bottom: parent.bottom
                    width: parent.width
                    height: 1
                    color: Theme.alpha(Theme.border, 0.6)
                }

                MouseArea {
                    id: rowMouse
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: root.rowClicked(row.index, row.modelData)
                }
            }
        }
    }

    EmptyState {
        anchors.centerIn: parent
        width: parent.width - Theme.space6 * 2
        visible: root.rows.length === 0
        icon: "list-checks"
        title: root.emptyText
    }
}
