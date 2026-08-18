// SectionLabel is the small uppercase heading used above every group in App
// Options and on the pages — the twin of the web flavor's repeated
// `text-xs font-semibold text-app-subtext uppercase tracking-wider`.
//
// It exists as a component because that class string appeared in a dozen places
// and one of them was always a shade off.

import QtQuick
import App

Txt {
    color: Theme.subtext
    pixelSize: Theme.fontXs
    weight: Font.DemiBold
    letterSpacing: 0.8
    capitalization: Font.AllUppercase
}
