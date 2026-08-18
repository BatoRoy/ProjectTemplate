pragma ComponentBehavior: Bound

// Masonry — the twin of the Masonry in components/layout/Responsive.tsx: a column flow
// where items keep their natural height and pack without a shared row baseline.
//
// CSS gets this free with `column-count`. QML has no equivalent, so the placement is done
// here: each item goes to whichever column is currently shortest, which is the standard
// greedy fill and produces the same result as CSS for items in source order.
//
// Takes a `model` and a `delegate` rather than arbitrary children, because the layout has
// to know each item's height before it can place the next one.

import QtQuick
import App

Item {
    id: root

    property int columns: 3
    property int spacing: Theme.space3
    property alias model: repeater.model
    property Component delegate: null

    readonly property real columnWidth: columns > 0 ? (width - spacing * (columns - 1)) / columns : width

    // Recomputed whenever anything that affects placement changes. Kept as one function
    // rather than per-item bindings: an item's position depends on every item before it,
    // so incremental bindings would cascade unpredictably.
    function relayout() {
        var heights = [];
        for (var c = 0; c < columns; ++c)
            heights.push(0);

        for (var i = 0; i < repeater.count; ++i) {
            var item = repeater.itemAt(i);
            if (!item)
                continue;

            var shortest = 0;
            for (var k = 1; k < columns; ++k)
                if (heights[k] < heights[shortest])
                    shortest = k;

            item.width = columnWidth;
            item.x = shortest * (columnWidth + spacing);
            item.y = heights[shortest];
            heights[shortest] += item.height + spacing;
        }

        var tallest = 0;
        for (var j = 0; j < heights.length; ++j)
            tallest = Math.max(tallest, heights[j]);
        root.implicitHeight = Math.max(0, tallest - spacing);
    }

    onWidthChanged: relayout()
    onColumnsChanged: relayout()
    onSpacingChanged: relayout()

    Repeater {
        id: repeater
        delegate: root.delegate
        onCountChanged: Qt.callLater(root.relayout)
        onItemAdded: (index, item) => {
            // An item's height often settles a frame after it is created, so relayout is
            // re-run when it changes rather than only once at creation.
            item.heightChanged.connect(root.relayout);
            Qt.callLater(root.relayout);
        }
    }
}
