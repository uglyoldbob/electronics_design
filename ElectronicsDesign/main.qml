import QtQuick 2.0
import QtQuick.Controls 2.12
import QtQuick.Layouts 1.12
import uglyoldbob 1.0

ApplicationWindow {
    visible: true
    id: window
    title: ((DesignSingleton.unsaved !== 0) ? "" : "Unsaved ") + "Design: " + DesignSingleton.title

    LibraryEditor {
        id: lib_edit
    }

    Button {
            text: "Ok"
            onClicked: {
                lib_edit.open();
            }
        }

}
