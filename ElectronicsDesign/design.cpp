#include "design.h"

#include <QFile>
#include <QTextStream>
#include <QQmlEngine>

design::design()
{
    saved_to_disk = 0;
    unsaved_changes = 0;
    title = "Untitled";
    emit unsaved_changed();
    emit title_changed();
}

design::~design()
{

}

void design::qml_register()
{
    qmlRegisterSingletonType<design>("uglyoldbob", 1, 0, "DesignSingleton", [](QQmlEngine *engine, QJSEngine *scriptEngine) -> QObject * {
        Q_UNUSED(engine)
        Q_UNUSED(scriptEngine)
        design *nd = new design();
        return nd;
    });
}

qint8 design::has_unsaved_changes() const
{
    return unsaved_changes;
}

void design::set_title(QString val)
{
    title = val;
    emit title_changed();
}

QString design::get_title() const
{
    return title;
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
