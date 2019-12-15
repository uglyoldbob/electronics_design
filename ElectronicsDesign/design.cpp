#include "design.h"

#include <QFile>
#include <QTextStream>
#include <QQmlEngine>

design::design()
{
    saved_to_disk = 0;
    unsaved_changes = 0;
    emit blab_changed();
    emit unsaved_changed();
}

design::~design()
{

}

void design::tweak_blabbing()
{
 blabbing = (blabbing?0:1);
 emit blab_changed();
}

void design::qml_register()
{
    qmlRegisterType<design>("uglyoldbob", 1, 0, "Design");
}

void design::newDesign()
{

}

void design::loadFromFile(QString filename)
{
    saveCopyAs(filename);
    saved_to_disk = 1;
}

bool design::hasFileName(void)
{
    return !filename.isEmpty();
}

void design::saveToFile(void)
{
    saveCopyAs(this->filename);
    saved_to_disk = 1;
}

void design::saveToFile(QString filename)
{
    this->filename = filename;
    saveToFile();
}

int design::saveCopyAs(QString copy)
{
    QFile f(copy);

    if (!f.open(QIODevice::WriteOnly | QIODevice::Text))
        return -1;

    QTextStream out(&f);
    out << "Design" << "\n";

    return 0;
}
