pragma ComponentBehavior: Bound

// PanelGroup — the twin of components/layout/PanelGroup.tsx: N panes with draggable
// dividers between them, nestable to build any split arrangement.
//
// ResizablePanels handles the two-pane case with a single fraction. This generalises it:
// sizes are weights that sum to whatever they sum to, so adding or removing a pane does
// not require renormalising the others, and a drag moves weight between exactly the two
// panes either side of the handle — the rest of the layout does not shift.
//
// Nesting is just a PanelGroup inside a Panel:
//
//     PanelGroup {
//         Panel { PanelGroup { vertical: true; Panel {…} Panel {…} } }
//         Panel {…}
//     }
//
// Sizes persist when `settingsKey` is set, which is what makes a split layout survive a
// restart the way an editor's does.

import QtQuick
import QtCore
import App

Item {
    id: root

    property bool vertical: false
    property int handleSize: Theme.px(5)
    // When set, weights are stored under this key in the app's ui.conf.
    property string settingsKey: ""

    default property alias panels: container.data

    readonly property real extent: vertical ? height : width
    readonly property var items: {
        var out = [];
        for (var i = 0; i < container.children.length; ++i) {
            var c = container.children[i];
            // Only Panels take part; anything else a caller drops in is left alone.
            if (c && c.minimumSize !== undefined)
                out.push(c);
        }
        return out;
    }

    // Weights live here rather than on the Panels so a drag is one assignment and the
    // whole group re-lays out from a single source.
    property var weights: []

    Component.onCompleted: {
        restore();
        relayout();
    }
    // Both dimensions, not just `extent`. A vertical group divides its height, so its
    // extent never changes when the width does — but its panels are still sized to the
    // full width, so a width change that does not trigger a relayout leaves every panel
    // at the old one. That is exactly what happens to a nested group when its parent
    // panel is resized along the other axis.
    onWidthChanged: relayout()
    onHeightChanged: relayout()
    onVerticalChanged: relayout()

    function totalWeight() {
        var t = 0;
        for (var i = 0; i < weights.length; ++i)
            t += weights[i];
        return t > 0 ? t : 1;
    }

    function restore() {
        var w = [];
        if (settingsKey !== "" && store.value(settingsKey, "") !== "") {
            try {
                w = JSON.parse(store.value(settingsKey, "[]"));
            } catch (e) {
                w = [];
            }
        }
        // A stored layout from before a pane was added or removed is discarded rather
        // than stretched: a stale weight list silently mis-sizes everything.
        if (w.length !== items.length) {
            w = [];
            for (var i = 0; i < items.length; ++i)
                w.push(items[i].weight);
        }
        weights = w;
    }

    function persist() {
        if (settingsKey !== "")
            store.setValue(settingsKey, JSON.stringify(weights));
    }

    function relayout() {
        var n = items.length;
        if (n === 0 || extent <= 0)
            return;
        if (weights.length !== n)
            restore();

        var handles = handleSize * (n - 1);
        var usable = Math.max(0, extent - handles);
        var total = totalWeight();

        // Sizes from the weights, floored at each panel's minimum.
        var sizes = [];
        var sum = 0;
        for (var i = 0; i < n; ++i) {
            sizes.push(Math.max(items[i].minimumSize, usable * weights[i] / total));
            sum += sizes[i];
        }

        // The minimums can add up to more than there is — two 80px panels and a handle
        // need 165px in a 150px box. Honouring them then overflows the group and pushes a
        // panel outside its parent, which is worse than being slightly under minimum. When
        // they do not fit, everything scales down together so the proportions survive.
        if (sum > usable && sum > 0) {
            var shrink = usable / sum;
            for (var k = 0; k < n; ++k)
                sizes[k] *= shrink;
        }

        var pos = 0;
        for (i = 0; i < n; ++i) {
            var size = sizes[i];
            var it = items[i];
            if (vertical) {
                it.x = 0;
                it.y = pos;
                it.width = root.width;
                it.height = size;
            } else {
                it.y = 0;
                it.x = pos;
                it.height = root.height;
                it.width = size;
            }
            pos += size + (i < n - 1 ? handleSize : 0);
        }
        handleRepeater.model = Math.max(0, n - 1);
    }

    // Move weight between the two panes either side of handle `i`, in pixels.
    function drag(i, delta) {
        var total = totalWeight();
        var usable = Math.max(1, extent - handleSize * (items.length - 1));
        var perPixel = total / usable;

        var a = weights[i] + delta * perPixel;
        var b = weights[i + 1] - delta * perPixel;

        var minA = items[i].minimumSize * perPixel;
        var minB = items[i + 1].minimumSize * perPixel;
        if (a < minA || b < minB)
            return;

        var next = weights.slice();
        next[i] = a;
        next[i + 1] = b;
        weights = next;
        relayout();
    }

    Settings {
        id: store
        category: "panels"
        location: StandardPaths.writableLocation(StandardPaths.ConfigLocation) + "/" + Brand.slug + "/ui.conf"
    }

    Item {
        id: container
        anchors.fill: parent
    }

    Repeater {
        id: handleRepeater
        model: 0

        delegate: Rectangle {
            id: handle
            required property int index

            readonly property var before: root.items[index]

            x: root.vertical ? 0 : (before ? before.x + before.width : 0)
            y: root.vertical ? (before ? before.y + before.height : 0) : 0
            width: root.vertical ? root.width : root.handleSize
            height: root.vertical ? root.handleSize : root.height

            color: handleMouse.containsMouse || handleMouse.pressed ? Theme.accentRing : Theme.border
            Behavior on color {
                ColorAnimation {
                    duration: Theme.animFast
                }
            }

            MouseArea {
                id: handleMouse
                anchors.fill: parent
                // A 5px divider is hard to grab; widen the target without widening the line.
                anchors.margins: -Theme.px(3)
                hoverEnabled: true
                cursorShape: root.vertical ? Qt.SizeVerCursor : Qt.SizeHorCursor

                property real last: 0
                onPressed: e => last = root.vertical ? e.y : e.x
                onPositionChanged: e => {
                    if (!pressed)
                        return;
                    var now = root.vertical ? e.y : e.x;
                    root.drag(handle.index, now - last);
                }
                onReleased: root.persist()
            }
        }
    }
}
