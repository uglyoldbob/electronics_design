#ifndef DESIGN_H_
#define DESIGN_H_

#include <QObject>
#include <QString>

class design : public QObject
{
Q_OBJECT
Q_PROPERTY(quint8 blab READ read_blab NOTIFY blab_changed)

public:
    Q_PROPERTY(qint8 unsaved READ has_unsaved_changes() NOTIFY unsaved_changed)
    explicit design();
    virtual ~design();
    qint8 has_unsaved_changes() const { return unsaved_changes; }
    quint8 read_blab() const { return blabbing; }

    void newDesign();
    int closeDesign();
    void loadFromFile(QString filename);
    void saveToFile(void);
    void saveToFile(QString filename);
    bool hasFileName(void);
    int saveCopyAs(QString copy);
    static void qml_register();

public slots:
    void tweak_blabbing();
signals:
    void unsaved_changed();
    void blab_changed();

private:
    QString filename;
    QString blab;
    qint8 unsaved_changes;
    int saved_to_disk;
    quint8 blabbing;
};

#endif
