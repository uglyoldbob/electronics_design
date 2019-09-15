#ifndef DESIGN_H_
#define DESIGN_H_

#include <QObject>
#include <QString>

class design : public QObject
{
Q_OBJECT

public:
    explicit design();
    virtual ~design();
    void loadFromFile(QString filename);
    void saveToFile(void);
    void saveToFile(QString filename);
    bool hasFileName(void);
    int saveCopyAs(QString copy);

private slots:
signals:
    void hasUnsavedChanges(bool n);

private:
    QString filename;
    int saved_to_disk;
};

#endif
