#ifndef DESIGN_H_
#define DESIGN_H_

#include <QObject>
#include <QString>

class design : public QObject
{
Q_OBJECT

public:
    Q_PROPERTY(qint8 unsaved READ has_unsaved_changes NOTIFY unsaved_changed)
    Q_PROPERTY(QString title READ get_title WRITE set_title NOTIFY title_changed)
    explicit design();
    virtual ~design();
    qint8 has_unsaved_changes() const;
    void set_title(QString val);
    QString get_title() const;

    void newDesign();
    int closeDesign();
    void loadFromFile(QString filename);
    void saveToFile(void);
    void saveToFile(QString filename);
    bool hasFileName(void);
    int saveCopyAs(QString copy);
    static void qml_register();

public slots:

signals:
    void unsaved_changed();
    void title_changed();

private:
    QString filename;
    QString title;  //the title of the current design
    qint8 unsaved_changes;
    int saved_to_disk;
};

#endif
