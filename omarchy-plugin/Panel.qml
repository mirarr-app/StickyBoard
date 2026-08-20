import QtQuick
import Quickshell
import Quickshell.Io
import qs.Commons
import qs.Ui

Panel {
  id: root
  moduleName: "stickyboard.notes"
  ipcTarget: "stickyboard.notes"
  manageIpc: false

  property var anchorItem: null
  property var hostWidget: null
  readonly property var barIdentity: hostWidget || root

  readonly property color contentForeground: bar ? bar.foreground : Color.foreground
  readonly property string contentFontFamily: bar ? bar.fontFamily : Style.font.family
  readonly property color dim: Qt.darker(contentForeground, 1.45)

  property string errorText: ""
  property bool submitting: false

  readonly property string helperPath: Qt.resolvedUrl("add-note.sh").toString().replace(/^file:\/\//, "")

  function open() {
    errorText = ""
    submitting = false
    root.controller.show()
    Qt.callLater(function() {
      if (root.opened) noteField.forceActiveFocus()
    })
  }

  function close() {
    submitting = false
    root.controller.hide()
  }

  function toggle() {
    if (root.opened) root.close()
    else root.open()
  }

  function switchPanel(direction) {
    if (root.bar && typeof root.bar.switchPanelFrom === "function")
      return root.bar.switchPanelFrom(root.barIdentity, direction)
    return false
  }

  function submit() {
    if (root.submitting) return
    var text = noteField.text.replace(/^\s+|\s+$/g, "")
    if (!text) {
      root.close()
      return
    }
    root.errorText = ""
    root.submitting = true
    createProc.command = [root.helperPath, text]
    createProc.running = true
  }

  Process {
    id: createProc
    stdout: StdioCollector { waitForEnd: true }
    stderr: StdioCollector { waitForEnd: true }
    onExited: function(exitCode) {
      root.submitting = false
      if (exitCode === 0) {
        noteField.text = ""
        root.errorText = ""
        root.close()
        return
      }
      var err = String(stderr.text || "").replace(/^\s+|\s+$/g, "")
      root.errorText = err.length ? err : "Could not create note. Is stickyboard-daemon running?"
    }
  }

  KeyboardPanel {
    id: panel
    anchorItem: root.anchorItem
    owner: root.barIdentity
    bar: root.bar
    open: root.opened
    focusTarget: noteField
    contentWidth: panel.fittedContentWidth(Style.space(380))
    contentHeight: panel.fittedContentHeight(mainColumn.implicitHeight)

    PanelKeyCatcher {
      id: keyCatcher
      anchors.fill: parent
      blocked: noteField.activeFocus
      onCloseRequested: root.close()
      onTabRequested: function(direction) { root.switchPanel(direction) }

      Column {
        id: mainColumn
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        spacing: Style.space(12)

        PanelHero {
          width: parent.width
          title: "New note"
          meta: "Saved onto workspace 6"
          foreground: root.contentForeground
          fontFamily: root.contentFontFamily
          iconComponent: Component {
            Text {
              text: "󰎞"
              color: Color.accent
              font.family: Style.font.family
              font.pixelSize: Style.font.display
            }
          }
        }

        TextField {
          id: noteField
          width: parent.width
          placeholderText: "What's on your mind?"
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.body
          foreground: root.contentForeground
          accent: Color.accent
          enabled: !root.submitting
          onAccepted: root.submit()
          Keys.onEscapePressed: root.close()
        }

        Text {
          width: parent.width
          visible: root.errorText.length > 0
          text: root.errorText
          wrapMode: Text.WordWrap
          color: Color.urgent
          font.family: root.contentFontFamily
          font.pixelSize: Style.font.caption
        }

        Row {
          width: parent.width
          spacing: Style.space(8)

          Text {
            anchors.verticalCenter: parent.verticalCenter
            width: parent.width - addButton.implicitWidth - parent.spacing
            text: root.submitting ? "Saving…" : "Enter to add  ·  Esc to close"
            color: root.dim
            font.family: root.contentFontFamily
            font.pixelSize: Style.font.caption
            elide: Text.ElideRight
          }

          Button {
            id: addButton
            text: "Add"
            enabled: !root.submitting && noteField.text.replace(/^\s+|\s+$/g, "").length > 0
            foreground: root.contentForeground
            accent: Color.accent
            onClicked: root.submit()
          }
        }
      }
    }
  }
}
