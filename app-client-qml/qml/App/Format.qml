pragma Singleton

// Format is the twin of what date-fns and a handful of inline helpers do on the React
// side: the formatting the UI needs, in one place.
//
// It exists as a singleton because BatoAI ended up defining gib() and humanBytes() three
// times in three components, and they had already started to disagree.
//
// Dates use JavaScript's own Date rather than a library: QML has no bundler, so a
// dependency here would have to be vendored, and the handful of operations the date
// components need are a few lines each.

import QtQuick

QtObject {
    readonly property var monthNames: [qsTr("January"), qsTr("February"), qsTr("March"), qsTr("April"), qsTr("May"), qsTr("June"), qsTr("July"), qsTr("August"), qsTr("September"), qsTr("October"), qsTr("November"), qsTr("December")]
    // Monday first, matching the web flavor's calendar.
    readonly property var weekdayShort: [qsTr("Mo"), qsTr("Tu"), qsTr("We"), qsTr("Th"), qsTr("Fr"), qsTr("Sa"), qsTr("Su")]

    function pad(n) {
        return n < 10 ? "0" + n : String(n);
    }

    function date(d) {
        if (!d)
            return "";
        return d.getFullYear() + "-" + pad(d.getMonth() + 1) + "-" + pad(d.getDate());
    }
    function time(d, seconds) {
        if (!d)
            return "";
        return pad(d.getHours()) + ":" + pad(d.getMinutes()) + (seconds ? ":" + pad(d.getSeconds()) : "");
    }
    function dateTime(d) {
        return d ? date(d) + " " + time(d) : "";
    }
    function monthYear(d) {
        return d ? monthNames[d.getMonth()] + " " + d.getFullYear() : "";
    }

    function sameDay(a, b) {
        return !!a && !!b && a.getFullYear() === b.getFullYear() && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
    }
    function startOfDay(d) {
        return new Date(d.getFullYear(), d.getMonth(), d.getDate());
    }
    function addMonths(d, n) {
        return new Date(d.getFullYear(), d.getMonth() + n, 1);
    }
    function daysInMonth(d) {
        return new Date(d.getFullYear(), d.getMonth() + 1, 0).getDate();
    }

    // Monday-based index of the 1st, so a month grid starts in the right column.
    function firstWeekday(d) {
        var js = new Date(d.getFullYear(), d.getMonth(), 1).getDay();  // 0 = Sunday
        return (js + 6) % 7;
    }

    // Sizes, in the suite's usual units.
    function bytes(n) {
        if (n === undefined || n === null)
            return "";
        var units = ["B", "KiB", "MiB", "GiB", "TiB"], i = 0, v = n;
        while (v >= 1024 && i < units.length - 1) {
            v /= 1024;
            i++;
        }
        return (i === 0 ? v : v.toFixed(1)) + " " + units[i];
    }
    function gib(mib) {
        return (mib / 1024).toFixed(1) + " GiB";
    }

    function duration(seconds) {
        var s = Math.max(0, Math.round(seconds));
        var h = Math.floor(s / 3600), m = Math.floor((s % 3600) / 60), r = s % 60;
        return h > 0 ? h + ":" + pad(m) + ":" + pad(r) : m + ":" + pad(r);
    }
}
