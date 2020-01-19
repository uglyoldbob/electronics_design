#ifndef LIBRARY_H
#define LIBRARY_H

#include <QFile>
#include <QtGlobal>

enum class LibraryLocations
{
    LocalFile
};

class library
{
public:
    library();

private:
    quint64 creator;    //unique identifier for the creator of the library
    quint64 id;         //library id, unique to the creator
    quint16 version_major;
    quint16 version_minor;
    LibraryLocations location;

    //may eventually break access into a separated class, with a subclass for each storage type
    QFile *file;
    void create_file(QString name);
};

#endif // LIBRARY_H
