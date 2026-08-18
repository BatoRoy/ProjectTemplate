pragma ComponentBehavior: Bound

// DataTable — the twin of components/data/DataTable.tsx: sortable columns, a global
// filter, row selection, pagination and per-row actions, with a sticky header.
//
// columns: [{ key, label, width?, align?, mono? }]
//   `width` is a share, not pixels — the columns always fill the table.
// rows:    [{ <key>: value, … }]
// rowActions: a function (row) → [{ label?, icon?, tone?, showOnHover?, onTriggered }]
//   Matching the React signature: a function rather than a list, because which actions a
//   row offers usually depends on the row.
//
// Sorting, filtering and paging are all done here rather than expected from the caller,
// as they are on the React side: a table that cannot do them pushes the same forty lines
// into every page that uses one. Set `sortable: false` / omit `filterable` / `pageSize: 0`
// for server-side handling.

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import App

Rectangle {
    id: root

    property var columns: []
    property var rows: []
    property bool sortable: true
    property bool filterable: false
    property bool selectable: false
    // Row indices into `rows` (the unsorted, unfiltered source), so a selection survives
    // sorting and paging.
    property var selected: []
    property int pageSize: 0            // 0 = no pagination
    property var rowActions: null
    property string sortKey: ""
    property bool sortDescending: false
    property string emptyText: qsTr("No rows")
    property string filterPlaceholder: qsTr("Filter…")

    signal rowClicked(int index, var row)
    signal selectionChanged(var selected)

    color: Theme.card
    border.color: Theme.border
    border.width: 1
    radius: Theme.radiusXl
    clip: true
    implicitHeight: Theme.px(280)

    property string query: ""
    property int page: 1

    // Each stage keeps the source index alongside the row, so selection and rowClicked
    // always refer to the caller's array rather than to a position in a derived view.
    readonly property var indexed: {
        var out = [];
        for (var i = 0; i < rows.length; ++i)
            out.push({
                i: i,
                row: rows[i]
            });
        return out;
    }

    readonly property var filtered: {
        var q = query.trim().toLowerCase();
        if (!filterable || q === "")
            return indexed;
        return indexed.filter(function (e) {
            for (var c = 0; c < root.columns.length; ++c) {
                var v = e.row[root.columns[c].key];
                if (v !== undefined && v !== null && String(v).toLowerCase().indexOf(q) !== -1)
                    return true;
            }
            return false;
        });
    }

    readonly property var sorted: {
        if (!sortKey || !sortable)
            return filtered;
        var copy = filtered.slice();
        copy.sort(function (a, b) {
            var x = a.row[root.sortKey], y = b.row[root.sortKey];
            if (x === y)
                return 0;
            // Numbers compare numerically, everything else as a localised string —
            // otherwise "10" sorts before "9".
            var r = (typeof x === "number" && typeof y === "number") ? x - y : String(x).localeCompare(String(y));
            return root.sortDescending ? -r : r;
        });
        return copy;
    }

    readonly property int pageCount: pageSize > 0 ? Math.max(1, Math.ceil(sorted.length / pageSize)) : 1
    readonly property var visibleRows: pageSize > 0 ? sorted.slice((Math.min(page, pageCount) - 1) * pageSize, Math.min(page, pageCount) * pageSize) : sorted

    // A filter that leaves fewer pages than the one you are on would otherwise show an
    // empty table with rows that exist.
    onFilteredChanged: page = 1

    // ── Selection ────────────────────────────────────────────────────────────
    function isSelected(i) {
        return selected.indexOf(i) !== -1;
    }
    function toggle(i) {
        var next = selected.slice();
        var at = next.indexOf(i);
        if (at === -1)
            next.push(i);
        else
            next.splice(at, 1);
        selected = next;
        root.selectionChanged(next);
    }
    // The header checkbox acts on what is *visible*, not on everything: selecting rows you
    // have filtered away is never what was meant.
    readonly property var visibleIndices: visibleRows.map(e => e.i)
    readonly property bool allVisibleSelected: visibleIndices.length > 0 && visibleIndices.every(i => root.isSelected(i))
    readonly property bool someVisibleSelected: visibleIndices.some(i => root.isSelected(i)) && !allVisibleSelected

    function toggleAllVisible() {
        var next = selected.slice();
        if (allVisibleSelected) {
            next = next.filter(i => root.visibleIndices.indexOf(i) === -1);
        } else {
            for (var k = 0; k < visibleIndices.length; ++k)
                if (next.indexOf(visibleIndices[k]) === -1)
                    next.push(visibleIndices[k]);
        }
        selected = next;
        root.selectionChanged(next);
    }

    readonly property int checkboxWidth: selectable ? Theme.px(34) : 0
    readonly property int actionsWidth: rowActions ? Theme.px(96) : 0

    function widthFor(col, total) {
        var shares = 0;
        for (var i = 0; i < root.columns.length; ++i)
            shares += root.columns[i].width ?? 1;
        return total * (col.width ?? 1) / shares;
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // ── Filter bar ───────────────────────────────────────────────────────
        Item {
            Layout.fillWidth: true
            implicitHeight: root.filterable ? Theme.px(46) : 0
            visible: root.filterable

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.space3
                anchors.rightMargin: Theme.space3
                spacing: Theme.space2

                Icon {
                    name: "search"
                    size: Theme.px(14)
                    color: Theme.muted
                }

                TextInput {
                    Layout.fillWidth: true
                    Layout.minimumWidth: 0
                    verticalAlignment: TextInput.AlignVCenter
                    color: Theme.text
                    font.family: Theme.fontSans
                    font.pixelSize: Theme.fontSm
                    selectByMouse: true
                    selectionColor: Theme.accentTintHi
                    clip: true
                    renderType: Text.NativeRendering
                    onTextChanged: root.query = text

                    Txt {
                        anchors.verticalCenter: parent.verticalCenter
                        visible: parent.text.length === 0
                        text: root.filterPlaceholder
                        color: Theme.muted
                        pixelSize: Theme.fontSm
                    }
                }

                Txt {
                    visible: root.query !== ""
                    text: qsTr("%1 of %2").arg(root.filtered.length).arg(root.rows.length)
                    pixelSize: Theme.fontXs
                    color: Theme.muted
                }
            }

            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: Theme.border
            }
        }

        // ── Header ───────────────────────────────────────────────────────────
        // Outside the ListView, which is what makes it sticky without a per-frame update.
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: Theme.px(36)
            color: Theme.composite(Theme.border, Theme.card, 0.35)

            RowLayout {
                anchors.fill: parent
                anchors.leftMargin: Theme.space3
                anchors.rightMargin: Theme.space3
                spacing: 0

                Item {
                    visible: root.selectable
                    Layout.preferredWidth: root.checkboxWidth
                    Layout.fillHeight: true

                    Checkbox {
                        anchors.verticalCenter: parent.verticalCenter
                        checked: root.allVisibleSelected
                        indeterminate: root.someVisibleSelected
                        onToggled: root.toggleAllVisible()
                    }
                }

                Repeater {
                    model: root.columns
                    delegate: Item {
                        id: head
                        required property var modelData
                        Layout.preferredWidth: root.widthFor(modelData, root.width - Theme.space3 * 2 - root.checkboxWidth - root.actionsWidth)
                        Layout.fillHeight: true

                        Row {
                            anchors.verticalCenter: parent.verticalCenter
                            anchors.left: head.modelData.align === "right" ? undefined : parent.left
                            anchors.right: head.modelData.align === "right" ? parent.right : undefined
                            anchors.rightMargin: head.modelData.align === "right" ? Theme.space2 : 0
                            spacing: Theme.space1

                            Txt {
                                anchors.verticalCenter: parent.verticalCenter
                                // Natural width, capped at what the column can spare for
                                // it. Filling the column instead pushes the sort glyph to
                                // the far edge, where it reads as belonging to the next
                                // heading rather than to this one.
                                width: Math.min(implicitWidth, Math.max(0, head.width - Theme.px(14)))
                                elide: Text.ElideRight
                                text: head.modelData.label ?? head.modelData.key
                                pixelSize: Theme.fontXs
                                weight: Font.DemiBold
                                color: root.sortKey === head.modelData.key ? Theme.accentText : Theme.subtext
                                capitalization: Font.AllUppercase
                                letterSpacing: 0.6
                            }

                            Icon {
                                anchors.verticalCenter: parent.verticalCenter
                                visible: root.sortable
                                // An inactive column shows the neutral glyph rather than
                                // nothing, so it is discoverable that it can be sorted.
                                name: root.sortKey !== head.modelData.key ? "chevrons-up-down" : root.sortDescending ? "chevron-down" : "chevron-up"
                                size: Theme.px(11)
                                color: root.sortKey === head.modelData.key ? Theme.accentText : Theme.muted
                                opacity: root.sortKey === head.modelData.key ? 1 : 0.5
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

                Item {
                    visible: root.rowActions !== null
                    Layout.preferredWidth: root.actionsWidth
                    Layout.fillHeight: true
                }
            }

            Rectangle {
                anchors.bottom: parent.bottom
                width: parent.width
                height: 1
                color: Theme.border
            }
        }

        // ── Rows ─────────────────────────────────────────────────────────────
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: root.visibleRows
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {}

            delegate: Rectangle {
                id: row
                required property var modelData
                readonly property int sourceIndex: modelData.i
                readonly property var rowData: modelData.row

                width: list.width
                height: Theme.rowHeight
                color: root.isSelected(sourceIndex) ? Theme.accentTint : rowMouse.containsMouse ? Theme.alpha(Theme.border, 0.4) : "transparent"

                RowLayout {
                    anchors.fill: parent
                    anchors.leftMargin: Theme.space3
                    anchors.rightMargin: Theme.space3
                    spacing: 0

                    Item {
                        visible: root.selectable
                        Layout.preferredWidth: root.checkboxWidth
                        Layout.fillHeight: true

                        Checkbox {
                            anchors.verticalCenter: parent.verticalCenter
                            checked: root.isSelected(row.sourceIndex)
                            onToggled: root.toggle(row.sourceIndex)
                        }
                    }

                    Repeater {
                        model: root.columns
                        delegate: Item {
                            id: cell
                            required property var modelData
                            Layout.preferredWidth: root.widthFor(modelData, root.width - Theme.space3 * 2 - root.checkboxWidth - root.actionsWidth)
                            Layout.fillHeight: true

                            Txt {
                                anchors.verticalCenter: parent.verticalCenter
                                anchors.left: cell.modelData.align === "right" ? undefined : parent.left
                                anchors.right: cell.modelData.align === "right" ? parent.right : undefined
                                anchors.rightMargin: cell.modelData.align === "right" ? Theme.space2 : 0
                                width: Math.max(0, cell.width - Theme.space2)
                                horizontalAlignment: cell.modelData.align === "right" ? Text.AlignRight : Text.AlignLeft
                                text: String(row.rowData[cell.modelData.key] ?? "")
                                pixelSize: Theme.fontSm
                                family: cell.modelData.mono === true ? Theme.fontMono : Theme.fontSans
                                color: Theme.text
                                elide: Text.ElideRight
                            }
                        }
                    }

                    // Per-row actions. Icon-only ones appear on hover, matching the React
                    // default — a table with five visible icons per row is unreadable.
                    Row {
                        visible: root.rowActions !== null
                        Layout.preferredWidth: root.actionsWidth
                        Layout.fillHeight: true
                        layoutDirection: Qt.RightToLeft
                        spacing: Theme.space1

                        Repeater {
                            model: root.rowActions ? root.rowActions(row.rowData) : []
                            delegate: Item {
                                id: action
                                required property var modelData
                                readonly property bool labeled: modelData.label !== undefined
                                readonly property bool hoverOnly: modelData.showOnHover ?? !labeled
                                readonly property color tone: {
                                    switch (modelData.tone) {
                                    case "accent":
                                        return Theme.accent;
                                    case "danger":
                                        return Theme.red;
                                    default:
                                        return Theme.subtext;
                                    }
                                }

                                anchors.verticalCenter: parent.verticalCenter
                                width: labeled ? actionLabel.implicitWidth + Theme.space3 : Theme.px(24)
                                height: Theme.px(24)
                                visible: !hoverOnly || rowMouse.containsMouse
                                opacity: visible ? 1 : 0
                                Behavior on opacity {
                                    NumberAnimation {
                                        duration: Theme.animFast
                                    }
                                }

                                Rectangle {
                                    anchors.fill: parent
                                    radius: Theme.radiusSm
                                    color: actionMouse.containsMouse ? Theme.composite(action.tone, Theme.card, 0.15) : action.labeled ? Theme.composite(action.tone, Theme.card, 0.10) : "transparent"
                                    border.width: action.labeled ? 1 : 0
                                    border.color: Theme.alpha(action.tone, 0.25)
                                }

                                Txt {
                                    id: actionLabel
                                    anchors.centerIn: parent
                                    visible: action.labeled
                                    text: action.modelData.label ?? ""
                                    pixelSize: Theme.fontXs
                                    weight: Font.Medium
                                    color: Theme.readableOn(action.tone, Theme.card)
                                }

                                Icon {
                                    anchors.centerIn: parent
                                    visible: !action.labeled
                                    name: action.modelData.icon ?? "more-horizontal"
                                    size: Theme.px(13)
                                    color: actionMouse.containsMouse ? Theme.readableOn(action.tone, Theme.card) : Theme.muted
                                }

                                MouseArea {
                                    id: actionMouse
                                    anchors.fill: parent
                                    hoverEnabled: true
                                    cursorShape: Qt.PointingHandCursor
                                    onClicked: if (action.modelData.onTriggered)
                                        action.modelData.onTriggered()
                                }
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
                    // Below the checkbox and the action buttons, so neither also fires the
                    // row's own click.
                    z: -1
                    onClicked: root.rowClicked(row.sourceIndex, row.rowData)
                }
            }
        }

        // ── Pagination ───────────────────────────────────────────────────────
        Item {
            Layout.fillWidth: true
            implicitHeight: root.pageSize > 0 && root.pageCount > 1 ? Theme.px(46) : 0
            visible: root.pageSize > 0 && root.pageCount > 1

            Rectangle {
                anchors.top: parent.top
                width: parent.width
                height: 1
                color: Theme.border
            }

            Pagination {
                anchors.centerIn: parent
                // Its implicit width, clamped — stretching it spreads the page buttons to
                // the far corners of the footer.
                width: Math.min(implicitWidth, parent.width - Theme.space3 * 2)
                page: root.page
                pages: root.pageCount
                onPicked: p => root.page = p
            }
        }
    }

    EmptyState {
        anchors.centerIn: parent
        width: parent.width - Theme.space6 * 2
        visible: root.sorted.length === 0
        icon: root.query !== "" ? "search" : "list-checks"
        title: root.query !== "" ? qsTr("Nothing matches") : root.emptyText
        subtitle: root.query !== "" ? qsTr("No row contains “%1”.").arg(root.query) : ""
    }
}
