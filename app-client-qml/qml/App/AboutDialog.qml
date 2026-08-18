// AboutDialog is the twin of components/AboutDialog.tsx: the app's mark, its name,
// its version and tagline.
//
// Where it necessarily differs: the web flavor's check-for-updates / restart-to-
// install button is driven by electron-updater, and a `type: "desktop"` release has
// no self-update mechanism at all — `apps/` is anonymously readable and ships a
// latest-linux.yml precisely so electron-updater can work, and `desktop/` is neither.
//
// So this *detects* rather than installs: it asks the registry what the latest
// published version is and, when that is newer, shows the one command that upgrades.
// Honest about what it can do, and still useful.

import QtQuick
import QtQuick.Layouts
import App

Modal {
    id: root
    title: qsTr("About")
    width: Math.min(parent ? parent.width - Theme.space6 * 2 : Theme.px(380), Theme.px(380))

    // "" | checking | current | available | error
    property string updateState: ""
    property string latestVersion: ""

    onOpened: checkForUpdate()

    ColumnLayout {
        Layout.fillWidth: true
        spacing: Theme.space3

        Image {
            Layout.alignment: Qt.AlignHCenter
            source: Brand.icon
            sourceSize.width: Theme.px(64)
            sourceSize.height: Theme.px(64)
            smooth: true
        }

        Txt {
            Layout.alignment: Qt.AlignHCenter
            text: Brand.appName
            color: Theme.text
            pixelSize: Theme.fontLg
            weight: Font.DemiBold
        }

        MonoText {
            Layout.alignment: Qt.AlignHCenter
            text: "v" + Brand.version
            color: Theme.muted
        }

        Txt {
            Layout.alignment: Qt.AlignHCenter
            Layout.fillWidth: true
            text: Brand.tagline
            color: Theme.subtext
            pixelSize: Theme.fontXs
            wrapMode: Text.WordWrap
            horizontalAlignment: Text.AlignHCenter
        }
    }

    // Update status
    ColumnLayout {
        Layout.fillWidth: true
        spacing: Theme.space2
        visible: root.updateState !== ""

        RowLayout {
            Layout.alignment: Qt.AlignHCenter
            spacing: Theme.space2

            Spinner {
                visible: root.updateState === "checking"
                size: Theme.px(13)
            }

            Txt {
                text: {
                    switch (root.updateState) {
                    case "checking":  return qsTr("Checking for updates…")
                    case "current":   return qsTr("Up to date")
                    case "available": return qsTr("v%1 is available").arg(root.latestVersion)
                    default:          return qsTr("Could not check for updates")
                    }
                }
                color: root.updateState === "available" ? Theme.accentText : Theme.muted
                pixelSize: Theme.fontXs
            }
        }

        // The upgrade path for a desktop release is `bato install`, so show exactly
        // that rather than a button that cannot do it.
        MonoText {
            Layout.alignment: Qt.AlignHCenter
            visible: root.updateState === "available"
            text: "bato install " + Brand.slug
            color: Theme.subtext
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.space2

        Item { Layout.fillWidth: true }

        Button {
            variant: "ghost"
            text: qsTr("Check again")
            visible: root.updateState === "current" || root.updateState === "error"
            onClicked: root.checkForUpdate()
        }

        Button {
            variant: "primary"
            text: qsTr("Close")
            onClicked: root.close()
        }
    }

    // The registry's catalog endpoint is public and read-only, so this needs no
    // credentials. The host forwards BATO_REGISTRY_URL — the same variable the bato
    // CLI reads — as --app-registry-url; without it there is nothing to ask and the
    // check is simply skipped rather than shown as an error.
    function checkForUpdate() {
        var registry = Env.get("APP_REGISTRY_URL", "")
        if (registry === "" || Brand.version === "dev") {
            root.updateState = ""
            return
        }
        root.updateState = "checking"
        Api.request("GET", registry.replace(/\/+$/, "") + "/api/apps/" + Brand.slug, null,
                    function (ok, data) {
                        if (!ok || !data || !data.latest) {
                            root.updateState = "error"
                            return
                        }
                        root.latestVersion = data.latest
                        root.updateState = isNewer(data.latest, Brand.version) ? "available" : "current"
                    })
    }

    function isNewer(a, b) {
        var pa = String(a).split("."), pb = String(b).split(".")
        for (var i = 0; i < 3; ++i) {
            var na = parseInt(pa[i] || "0", 10), nb = parseInt(pb[i] || "0", 10)
            if (na !== nb) return na > nb
        }
        return false
    }
}
