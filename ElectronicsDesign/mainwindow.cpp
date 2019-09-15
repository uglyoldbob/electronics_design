#include "mainwindow.h"
#include "ui_mainwindow.h"

#include <QFileDialog>

MainWindow::MainWindow(QWidget *parent) :
    QMainWindow(parent),
    ui(new Ui::MainWindow)
{
    d1 = nullptr; //initialize to null pointer
    ui->setupUi(this);
}

MainWindow::~MainWindow()
{
    if (d1 != nullptr)
    {
        delete d1;
        d1 = nullptr;
    }
    delete ui;
}

void MainWindow::on_pushButton_clicked()
{
    if (d1 == nullptr)
    {
        //TODO: find a better default folder
        QString fileName = QFileDialog::getOpenFileName(this,
            tr("Open Design"), "./", tr("Design Files (*.dsg)"));
        if (fileName.isNull())
        {
            emit hasDesign(false);
        }
        else
        {
            d1 = new design();
            emit hasDesign(true);
        }
    }
}

void MainWindow::on_pushButton_2_clicked()
{
    if (d1 == nullptr)
    {
        d1 = new design();
        emit hasDesign(true);
    }
}

void MainWindow::on_pushButton_3_clicked()
{
    if (d1 != nullptr)
    {
        delete d1;
        d1 = nullptr;
        emit hasDesign(false);
    }
}

void MainWindow::on_pushButton_4_clicked()
{
    if (d1 != nullptr)
    {
        if (d1->hasFileName())
        {
            d1->saveToFile();
        }
        else
        {
            //TODO: find a better default folder
            QString fileName = QFileDialog::getSaveFileName(this,
                tr("Save Design"), "./", tr("Design Files (*.dsg)"));
            if (!fileName.isEmpty())
            {
                d1->saveToFile(fileName);
            }
        }
    }
}
