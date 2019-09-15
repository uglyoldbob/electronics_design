#include "design.h"

#include <QFile>
#include <QTextStream>

design::design()
{
    saved_to_disk = 0;
    emit hasUnsavedChanges(true);
}

design::~design()
{

}

void design::loadFromFile(QString filename)
{
    saveCopyAs(filename);
    emit hasUnsavedChanges(false);
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
