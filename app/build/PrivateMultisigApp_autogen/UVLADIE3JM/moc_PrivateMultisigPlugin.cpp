/****************************************************************************
** Meta object code from reading C++ file 'PrivateMultisigPlugin.h'
**
** Created by: The Qt Meta Object Compiler version 69 (Qt 6.11.2)
**
** WARNING! All changes made in this file will be lost!
*****************************************************************************/

#include "../../../src/PrivateMultisigPlugin.h"
#include <QtCore/qmetatype.h>
#include <QtCore/qplugin.h>

#include <QtCore/qtmochelpers.h>

#include <memory>


#include <QtCore/qxptype_traits.h>
#if !defined(Q_MOC_OUTPUT_REVISION)
#error "The header file 'PrivateMultisigPlugin.h' doesn't include <QObject>."
#elif Q_MOC_OUTPUT_REVISION != 69
#error "This file was generated using the moc from 6.11.2. It"
#error "cannot be used with the include files from this version of Qt."
#error "(The moc has changed too much.)"
#endif

#ifndef Q_CONSTINIT
#define Q_CONSTINIT
#endif

QT_WARNING_PUSH
QT_WARNING_DISABLE_DEPRECATED
QT_WARNING_DISABLE_GCC("-Wuseless-cast")
namespace {
struct qt_meta_tag_ZN21PrivateMultisigPluginE_t {};
} // unnamed namespace

template <> constexpr inline auto PrivateMultisigPlugin::qt_create_metaobjectdata<qt_meta_tag_ZN21PrivateMultisigPluginE_t>()
{
    namespace QMC = QtMocConstants;
    QtMocHelpers::StringRefStorage qt_stringData {
        "PrivateMultisigPlugin",
        "initLogos",
        "",
        "LogosAPI*",
        "api"
    };

    QtMocHelpers::UintData qt_methods {
        // Method 'initLogos'
        QtMocHelpers::MethodData<void(LogosAPI *)>(1, 2, QMC::AccessPublic, QMetaType::Void, {{
            { 0x80000000 | 3, 4 },
        }}),
    };
    QtMocHelpers::UintData qt_properties {
    };
    QtMocHelpers::UintData qt_enums {
    };
    return QtMocHelpers::metaObjectData<PrivateMultisigPlugin, qt_meta_tag_ZN21PrivateMultisigPluginE_t>(QMC::MetaObjectFlag{}, qt_stringData,
            qt_methods, qt_properties, qt_enums);
}
Q_CONSTINIT const QMetaObject PrivateMultisigPlugin::staticMetaObject = { {
    QMetaObject::SuperData::link<QObject::staticMetaObject>(),
    qt_staticMetaObjectStaticContent<qt_meta_tag_ZN21PrivateMultisigPluginE_t>.stringdata,
    qt_staticMetaObjectStaticContent<qt_meta_tag_ZN21PrivateMultisigPluginE_t>.data,
    qt_static_metacall,
    nullptr,
    qt_staticMetaObjectRelocatingContent<qt_meta_tag_ZN21PrivateMultisigPluginE_t>.metaTypes,
    nullptr
} };

void PrivateMultisigPlugin::qt_static_metacall(QObject *_o, QMetaObject::Call _c, int _id, void **_a)
{
    auto *_t = static_cast<PrivateMultisigPlugin *>(_o);
    if (_c == QMetaObject::InvokeMetaMethod) {
        switch (_id) {
        case 0: _t->initLogos((*reinterpret_cast<std::add_pointer_t<LogosAPI*>>(_a[1]))); break;
        default: ;
        }
    }
}

const QMetaObject *PrivateMultisigPlugin::metaObject() const
{
    return QObject::d_ptr->metaObject ? QObject::d_ptr->dynamicMetaObject() : &staticMetaObject;
}

void *PrivateMultisigPlugin::qt_metacast(const char *_clname)
{
    if (!_clname) return nullptr;
    if (!strcmp(_clname, qt_staticMetaObjectStaticContent<qt_meta_tag_ZN21PrivateMultisigPluginE_t>.strings))
        return static_cast<void*>(this);
    if (!strcmp(_clname, "IComponent"))
        return static_cast< IComponent*>(this);
    if (!strcmp(_clname, "com.logos.component.IComponent"))
        return static_cast< IComponent*>(this);
    return QObject::qt_metacast(_clname);
}

int PrivateMultisigPlugin::qt_metacall(QMetaObject::Call _c, int _id, void **_a)
{
    _id = QObject::qt_metacall(_c, _id, _a);
    if (_id < 0)
        return _id;
    if (_c == QMetaObject::InvokeMetaMethod) {
        if (_id < 1)
            qt_static_metacall(this, _c, _id, _a);
        _id -= 1;
    }
    if (_c == QMetaObject::RegisterMethodArgumentMetaType) {
        if (_id < 1)
            *reinterpret_cast<QMetaType *>(_a[0]) = QMetaType();
        _id -= 1;
    }
    return _id;
}

#ifdef QT_MOC_EXPORT_PLUGIN_V2
static constexpr unsigned char qt_pluginMetaDataV2_PrivateMultisigPlugin[] = {
    0xbf, 
    // "IID"
    0x02,  0x78,  0x1e,  'c',  'o',  'm',  '.',  'l', 
    'o',  'g',  'o',  's',  '.',  'c',  'o',  'm', 
    'p',  'o',  'n',  'e',  'n',  't',  '.',  'I', 
    'C',  'o',  'm',  'p',  'o',  'n',  'e',  'n', 
    't', 
    // "className"
    0x03,  0x75,  'P',  'r',  'i',  'v',  'a',  't', 
    'e',  'M',  'u',  'l',  't',  'i',  's',  'i', 
    'g',  'P',  'l',  'u',  'g',  'i',  'n', 
    // "MetaData"
    0x04,  0xac,  0x66,  'a',  'u',  't',  'h',  'o', 
    'r',  0x6a,  'p',  'r',  'a',  'm',  'a',  'd', 
    'a',  'n',  'i',  'f',  0x68,  'c',  'a',  't', 
    'e',  'g',  'o',  'r',  'y',  0x65,  't',  'o', 
    'o',  'l',  's',  0x6c,  'd',  'e',  'p',  'e', 
    'n',  'd',  'e',  'n',  'c',  'i',  'e',  's', 
    0x80,  0x6b,  'd',  'e',  's',  'c',  'r',  'i', 
    'p',  't',  'i',  'o',  'n',  0x78,  0x81,  'P', 
    'r',  'i',  'v',  'a',  't',  'e',  ' ',  'M', 
    '-',  'o',  'f',  '-',  'N',  ' ',  'm',  'u', 
    'l',  't',  'i',  's',  'i',  'g',  ' ',  'f', 
    'o',  'r',  ' ',  'L',  'E',  'Z',  ':',  ' ', 
    's',  'h',  'i',  'e',  'l',  'd',  'e',  'd', 
    ' ',  'm',  'e',  'm',  'b',  'e',  'r',  's', 
    ' ',  'a',  'p',  'p',  'r',  'o',  'v',  'e', 
    ' ',  'w',  'i',  't',  'h',  'o',  'u',  't', 
    ' ',  'r',  'e',  'v',  'e',  'a',  'l',  'i', 
    'n',  'g',  ' ',  'w',  'h',  'i',  'c',  'h', 
    ' ',  'm',  'e',  'm',  'b',  'e',  'r',  ' ', 
    'a',  'p',  'p',  'r',  'o',  'v',  'e',  'd', 
    '.',  ' ',  'T',  'h',  'r',  'e',  's',  'h', 
    'o',  'l',  'd',  '-',  'o',  'n',  'l',  'y', 
    ' ',  'o',  'n',  '-',  'c',  'h',  'a',  'i', 
    'n',  ' ',  's',  't',  'a',  't',  'e',  '.', 
    0x68,  'h',  'o',  'm',  'e',  'p',  'a',  'g', 
    'e',  0x78,  0x24,  'h',  't',  't',  'p',  's', 
    ':',  '/',  '/',  'g',  'i',  't',  'h',  'u', 
    'b',  '.',  'c',  'o',  'm',  '/',  'p',  'r', 
    'a',  'm',  'a',  'd',  'a',  'n',  'i',  'f', 
    '/',  'l',  'p',  '0',  '0',  '0',  '2',  0x64, 
    'i',  'c',  'o',  'n',  0x68,  'i',  'c',  'o', 
    'n',  '.',  's',  'v',  'g',  0x67,  'l',  'i', 
    'c',  'e',  'n',  's',  'e',  0x71,  'M',  'I', 
    'T',  ' ',  'O',  'R',  ' ',  'A',  'p',  'a', 
    'c',  'h',  'e',  '-',  '2',  '.',  '0',  0x64, 
    'm',  'a',  'i',  'n',  0xa6,  0x6c,  'd',  'a', 
    'r',  'w',  'i',  'n',  '-',  'a',  'm',  'd', 
    '6',  '4',  0x78,  0x20,  'l',  'i',  'b',  'p', 
    'r',  'i',  'v',  'a',  't',  'e',  '_',  'm', 
    'u',  'l',  't',  'i',  's',  'i',  'g',  '_', 
    'p',  'l',  'u',  'g',  'i',  'n',  '.',  'd', 
    'y',  'l',  'i',  'b',  0x6c,  'd',  'a',  'r', 
    'w',  'i',  'n',  '-',  'a',  'r',  'm',  '6', 
    '4',  0x78,  0x20,  'l',  'i',  'b',  'p',  'r', 
    'i',  'v',  'a',  't',  'e',  '_',  'm',  'u', 
    'l',  't',  'i',  's',  'i',  'g',  '_',  'p', 
    'l',  'u',  'g',  'i',  'n',  '.',  'd',  'y', 
    'l',  'i',  'b',  0x6b,  'l',  'i',  'n',  'u', 
    'x',  '-',  'a',  'm',  'd',  '6',  '4',  0x78, 
    0x1d,  'l',  'i',  'b',  'p',  'r',  'i',  'v', 
    'a',  't',  'e',  '_',  'm',  'u',  'l',  't', 
    'i',  's',  'i',  'g',  '_',  'p',  'l',  'u', 
    'g',  'i',  'n',  '.',  's',  'o',  0x6f,  'l', 
    'i',  'n',  'u',  'x',  '-',  'a',  'm',  'd', 
    '6',  '4',  '-',  'd',  'e',  'v',  0x78,  0x1d, 
    'l',  'i',  'b',  'p',  'r',  'i',  'v',  'a', 
    't',  'e',  '_',  'm',  'u',  'l',  't',  'i', 
    's',  'i',  'g',  '_',  'p',  'l',  'u',  'g', 
    'i',  'n',  '.',  's',  'o',  0x6b,  'l',  'i', 
    'n',  'u',  'x',  '-',  'a',  'r',  'm',  '6', 
    '4',  0x78,  0x1d,  'l',  'i',  'b',  'p',  'r', 
    'i',  'v',  'a',  't',  'e',  '_',  'm',  'u', 
    'l',  't',  'i',  's',  'i',  'g',  '_',  'p', 
    'l',  'u',  'g',  'i',  'n',  '.',  's',  'o', 
    0x70,  'l',  'i',  'n',  'u',  'x',  '-',  'x', 
    '8',  '6',  '_',  '6',  '4',  '-',  'd',  'e', 
    'v',  0x78,  0x1d,  'l',  'i',  'b',  'p',  'r', 
    'i',  'v',  'a',  't',  'e',  '_',  'm',  'u', 
    'l',  't',  'i',  's',  'i',  'g',  '_',  'p', 
    'l',  'u',  'g',  'i',  'n',  '.',  's',  'o', 
    0x6f,  'm',  'a',  'n',  'i',  'f',  'e',  's', 
    't',  'V',  'e',  'r',  's',  'i',  'o',  'n', 
    0x65,  '0',  '.',  '2',  '.',  '0',  0x64,  'n', 
    'a',  'm',  'e',  0x70,  'p',  'r',  'i',  'v', 
    'a',  't',  'e',  '_',  'm',  'u',  'l',  't', 
    'i',  's',  'i',  'g',  0x64,  't',  'y',  'p', 
    'e',  0x62,  'u',  'i',  0x67,  'v',  'e',  'r', 
    's',  'i',  'o',  'n',  0x65,  '0',  '.',  '1', 
    '.',  '0', 
    0xff, 
};
QT_MOC_EXPORT_PLUGIN_V2(PrivateMultisigPlugin, PrivateMultisigPlugin, qt_pluginMetaDataV2_PrivateMultisigPlugin)
#else
QT_PLUGIN_METADATA_SECTION
Q_CONSTINIT static constexpr unsigned char qt_pluginMetaData_PrivateMultisigPlugin[] = {
    'Q', 'T', 'M', 'E', 'T', 'A', 'D', 'A', 'T', 'A', ' ', '!',
    // metadata version, Qt version, architectural requirements
    0, QT_VERSION_MAJOR, QT_VERSION_MINOR, qPluginArchRequirements(),
    0xbf, 
    // "IID"
    0x02,  0x78,  0x1e,  'c',  'o',  'm',  '.',  'l', 
    'o',  'g',  'o',  's',  '.',  'c',  'o',  'm', 
    'p',  'o',  'n',  'e',  'n',  't',  '.',  'I', 
    'C',  'o',  'm',  'p',  'o',  'n',  'e',  'n', 
    't', 
    // "className"
    0x03,  0x75,  'P',  'r',  'i',  'v',  'a',  't', 
    'e',  'M',  'u',  'l',  't',  'i',  's',  'i', 
    'g',  'P',  'l',  'u',  'g',  'i',  'n', 
    // "MetaData"
    0x04,  0xac,  0x66,  'a',  'u',  't',  'h',  'o', 
    'r',  0x6a,  'p',  'r',  'a',  'm',  'a',  'd', 
    'a',  'n',  'i',  'f',  0x68,  'c',  'a',  't', 
    'e',  'g',  'o',  'r',  'y',  0x65,  't',  'o', 
    'o',  'l',  's',  0x6c,  'd',  'e',  'p',  'e', 
    'n',  'd',  'e',  'n',  'c',  'i',  'e',  's', 
    0x80,  0x6b,  'd',  'e',  's',  'c',  'r',  'i', 
    'p',  't',  'i',  'o',  'n',  0x78,  0x81,  'P', 
    'r',  'i',  'v',  'a',  't',  'e',  ' ',  'M', 
    '-',  'o',  'f',  '-',  'N',  ' ',  'm',  'u', 
    'l',  't',  'i',  's',  'i',  'g',  ' ',  'f', 
    'o',  'r',  ' ',  'L',  'E',  'Z',  ':',  ' ', 
    's',  'h',  'i',  'e',  'l',  'd',  'e',  'd', 
    ' ',  'm',  'e',  'm',  'b',  'e',  'r',  's', 
    ' ',  'a',  'p',  'p',  'r',  'o',  'v',  'e', 
    ' ',  'w',  'i',  't',  'h',  'o',  'u',  't', 
    ' ',  'r',  'e',  'v',  'e',  'a',  'l',  'i', 
    'n',  'g',  ' ',  'w',  'h',  'i',  'c',  'h', 
    ' ',  'm',  'e',  'm',  'b',  'e',  'r',  ' ', 
    'a',  'p',  'p',  'r',  'o',  'v',  'e',  'd', 
    '.',  ' ',  'T',  'h',  'r',  'e',  's',  'h', 
    'o',  'l',  'd',  '-',  'o',  'n',  'l',  'y', 
    ' ',  'o',  'n',  '-',  'c',  'h',  'a',  'i', 
    'n',  ' ',  's',  't',  'a',  't',  'e',  '.', 
    0x68,  'h',  'o',  'm',  'e',  'p',  'a',  'g', 
    'e',  0x78,  0x24,  'h',  't',  't',  'p',  's', 
    ':',  '/',  '/',  'g',  'i',  't',  'h',  'u', 
    'b',  '.',  'c',  'o',  'm',  '/',  'p',  'r', 
    'a',  'm',  'a',  'd',  'a',  'n',  'i',  'f', 
    '/',  'l',  'p',  '0',  '0',  '0',  '2',  0x64, 
    'i',  'c',  'o',  'n',  0x68,  'i',  'c',  'o', 
    'n',  '.',  's',  'v',  'g',  0x67,  'l',  'i', 
    'c',  'e',  'n',  's',  'e',  0x71,  'M',  'I', 
    'T',  ' ',  'O',  'R',  ' ',  'A',  'p',  'a', 
    'c',  'h',  'e',  '-',  '2',  '.',  '0',  0x64, 
    'm',  'a',  'i',  'n',  0xa6,  0x6c,  'd',  'a', 
    'r',  'w',  'i',  'n',  '-',  'a',  'm',  'd', 
    '6',  '4',  0x78,  0x20,  'l',  'i',  'b',  'p', 
    'r',  'i',  'v',  'a',  't',  'e',  '_',  'm', 
    'u',  'l',  't',  'i',  's',  'i',  'g',  '_', 
    'p',  'l',  'u',  'g',  'i',  'n',  '.',  'd', 
    'y',  'l',  'i',  'b',  0x6c,  'd',  'a',  'r', 
    'w',  'i',  'n',  '-',  'a',  'r',  'm',  '6', 
    '4',  0x78,  0x20,  'l',  'i',  'b',  'p',  'r', 
    'i',  'v',  'a',  't',  'e',  '_',  'm',  'u', 
    'l',  't',  'i',  's',  'i',  'g',  '_',  'p', 
    'l',  'u',  'g',  'i',  'n',  '.',  'd',  'y', 
    'l',  'i',  'b',  0x6b,  'l',  'i',  'n',  'u', 
    'x',  '-',  'a',  'm',  'd',  '6',  '4',  0x78, 
    0x1d,  'l',  'i',  'b',  'p',  'r',  'i',  'v', 
    'a',  't',  'e',  '_',  'm',  'u',  'l',  't', 
    'i',  's',  'i',  'g',  '_',  'p',  'l',  'u', 
    'g',  'i',  'n',  '.',  's',  'o',  0x6f,  'l', 
    'i',  'n',  'u',  'x',  '-',  'a',  'm',  'd', 
    '6',  '4',  '-',  'd',  'e',  'v',  0x78,  0x1d, 
    'l',  'i',  'b',  'p',  'r',  'i',  'v',  'a', 
    't',  'e',  '_',  'm',  'u',  'l',  't',  'i', 
    's',  'i',  'g',  '_',  'p',  'l',  'u',  'g', 
    'i',  'n',  '.',  's',  'o',  0x6b,  'l',  'i', 
    'n',  'u',  'x',  '-',  'a',  'r',  'm',  '6', 
    '4',  0x78,  0x1d,  'l',  'i',  'b',  'p',  'r', 
    'i',  'v',  'a',  't',  'e',  '_',  'm',  'u', 
    'l',  't',  'i',  's',  'i',  'g',  '_',  'p', 
    'l',  'u',  'g',  'i',  'n',  '.',  's',  'o', 
    0x70,  'l',  'i',  'n',  'u',  'x',  '-',  'x', 
    '8',  '6',  '_',  '6',  '4',  '-',  'd',  'e', 
    'v',  0x78,  0x1d,  'l',  'i',  'b',  'p',  'r', 
    'i',  'v',  'a',  't',  'e',  '_',  'm',  'u', 
    'l',  't',  'i',  's',  'i',  'g',  '_',  'p', 
    'l',  'u',  'g',  'i',  'n',  '.',  's',  'o', 
    0x6f,  'm',  'a',  'n',  'i',  'f',  'e',  's', 
    't',  'V',  'e',  'r',  's',  'i',  'o',  'n', 
    0x65,  '0',  '.',  '2',  '.',  '0',  0x64,  'n', 
    'a',  'm',  'e',  0x70,  'p',  'r',  'i',  'v', 
    'a',  't',  'e',  '_',  'm',  'u',  'l',  't', 
    'i',  's',  'i',  'g',  0x64,  't',  'y',  'p', 
    'e',  0x62,  'u',  'i',  0x67,  'v',  'e',  'r', 
    's',  'i',  'o',  'n',  0x65,  '0',  '.',  '1', 
    '.',  '0', 
    0xff, 
};
QT_MOC_EXPORT_PLUGIN(PrivateMultisigPlugin, PrivateMultisigPlugin)
#endif  // QT_MOC_EXPORT_PLUGIN_V2

QT_WARNING_POP
