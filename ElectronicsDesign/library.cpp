#include "library.h"

library::library()
{
    file = nullptr;
}


void library::create_file(QString name)
{
    if (file == nullptr)
    {
        file = new QFile(name);
        if (file->open(QIODevice::NewOnly))
        {

        }
    }
}
