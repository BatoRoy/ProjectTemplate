// CodeEditor — the twin of components/inputs/CodeEditor.tsx.
//
// **The largest deliberate deviation in the kit, recorded in PARITY.md.** The web flavor
// is CodeMirror 6 with real language grammars, bracket matching, folding and an optional
// vim mode. None of that exists for QML, and vendoring an editor is out of scope for a
// template — so this is a monospace TextArea with line numbers and regex-based
// highlighting: keywords, strings, numbers and comments.
//
// Good enough to show a config file, a query or a snippet. Not an IDE. If an app needs
// one, that app should embed a real editor rather than the template pretending to.

import QtQuick
import QtQuick.Controls
import App

Rectangle {
    id: root

    // The plain source is the property callers read and write; `edit` shows either it or
    // a highlighted rendering of it, depending on focus.
    property string text: ""
    property string language: "javascript"   // javascript | json | go | plain
    property bool lineNumbers: true
    property bool readOnly: false
    signal edited

    implicitHeight: Theme.px(200)
    color: Theme.isLight ? Theme.surface : Theme.bg
    border.width: 1
    border.color: edit.activeFocus ? Theme.accentRing : Theme.border
    radius: Theme.radius
    clip: true

    readonly property var keywords: ({
            "javascript": ["const", "let", "var", "function", "return", "if", "else", "for", "while", "new", "class", "import", "export", "from", "async", "await", "true", "false", "null", "undefined"],
            "go": ["package", "import", "func", "return", "if", "else", "for", "range", "var", "const", "type", "struct", "interface", "go", "defer", "chan", "map", "nil", "true", "false"],
            "json": ["true", "false", "null"],
            "plain": []
        })

    // Qt's rich text is the only styling hook a TextEdit gives, so highlighting is done by
    // wrapping matches in <span>. Everything must be escaped first or a stray < in the
    // source silently eats the rest of the document.
    function escapeHtml(s) {
        return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }

    function highlight(src) {
        var out = escapeHtml(src);
        var kw = keywords[language] ?? [];
        if (kw.length > 0) {
            var re = new RegExp("\\b(" + kw.join("|") + ")\\b", "g");
            out = out.replace(re, '<span style="color:' + Theme.accentText + '">$1</span>');
        }
        out = out.replace(/(&quot;|")([^"]*)("|&quot;)/g, '<span style="color:' + Theme.green + '">$1$2$3</span>');
        out = out.replace(/\b(\d+(\.\d+)?)\b/g, '<span style="color:' + Theme.yellow + '">$1</span>');
        out = out.replace(/(\/\/[^\n]*)/g, '<span style="color:' + Theme.muted + '">$1</span>');
        return out;
    }

    Row {
        anchors.fill: parent
        anchors.margins: Theme.space2
        spacing: Theme.space2

        // Gutter. Its own column so the numbers do not scroll horizontally with the code.
        Column {
            visible: root.lineNumbers
            width: root.lineNumbers ? gutterMetrics.width : 0

            Repeater {
                model: edit.lineCount
                delegate: Txt {
                    required property int index
                    text: index + 1
                    family: Theme.fontMono
                    pixelSize: Theme.fontXs
                    color: Theme.muted
                    horizontalAlignment: Text.AlignRight
                    width: parent.width
                }
            }
        }

        TextMetrics {
            id: gutterMetrics
            font.family: Theme.fontMono
            font.pixelSize: Theme.fontXs
            text: String(Math.max(99, edit.lineCount))
        }

        Flickable {
            width: parent.width - (root.lineNumbers ? gutterMetrics.width + Theme.space2 : 0)
            height: parent.height
            contentWidth: edit.implicitWidth
            contentHeight: edit.implicitHeight
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {}

            TextEdit {
                id: edit
                width: Math.max(parent.width, implicitWidth)
                color: Theme.text
                font.family: Theme.fontMono
                font.pixelSize: Theme.fontXs
                selectByMouse: true
                selectionColor: Theme.accentTintHi
                selectedTextColor: Theme.text
                readOnly: root.readOnly
                renderType: Text.NativeRendering
                // Plain while focused, highlighted when not: re-parsing rich text on every
                // keystroke fights the cursor, which is the same reason CurrencyInput
                // shows a raw number while editing.
                // Plain while focused, highlighted when not. Re-parsing rich text on
                // every keystroke fights the cursor — the same reason CurrencyInput shows
                // a raw number while editing — so the swap happens on focus change and
                // the plain source is kept in `root.text` either way.
                textFormat: activeFocus ? TextEdit.PlainText : TextEdit.RichText
                text: activeFocus ? root.text : root.highlight(root.text)

                onActiveFocusChanged: {
                    if (!activeFocus) {
                        // getText() returns the plain characters regardless of the format
                        // currently being rendered, which is what makes the round trip safe.
                        root.text = getText(0, length);
                    }
                }
                onTextChanged: if (activeFocus) {
                    root.text = text;
                    root.edited();
                }
            }
        }
    }
}
