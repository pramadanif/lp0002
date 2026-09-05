/****************************************************************************
** Meta object code from reading C++ file 'PrivateMultisigBackend.h'
**
** Created by: The Qt Meta Object Compiler version 69 (Qt 6.11.2)
**
** WARNING! All changes made in this file will be lost!
*****************************************************************************/

#include "../../../src/PrivateMultisigBackend.h"
#include <QtCore/qmetatype.h>

#include <QtCore/qtmochelpers.h>

#include <memory>


#include <QtCore/qxptype_traits.h>
#if !defined(Q_MOC_OUTPUT_REVISION)
#error "The header file 'PrivateMultisigBackend.h' doesn't include <QObject>."
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
struct qt_meta_tag_ZN22PrivateMultisigBackendE_t {};
} // unnamed namespace

template <> constexpr inline auto PrivateMultisigBackend::qt_create_metaobjectdata<qt_meta_tag_ZN22PrivateMultisigBackendE_t>()
{
    namespace QMC = QtMocConstants;
    QtMocHelpers::StringRefStorage qt_stringData {
        "PrivateMultisigBackend",
        "configChanged",
        "",
        "proposalChanged",
        "busyChanged",
        "lastErrorChanged",
        "lastTxHashChanged",
        "lastResultChanged",
        "operationSuccess",
        "operation",
        "txHash",
        "operationError",
        "error",
        "walletPathChanged",
        "sequencerUrlChanged",
        "programIdHexChanged",
        "connectionStatusChanged",
        "walletAccountsChanged",
        "walletAccountInfoChanged",
        "walletDecodedAccountChanged",
        "setWalletPath",
        "v",
        "setSequencerUrl",
        "setProgramIdHex",
        "createMultisig",
        "creatorId",
        "configHash",
        "memberRoot",
        "m",
        "n",
        "multisigId",
        "QVariantList",
        "membershipProgramId",
        "createProposal",
        "proposerId",
        "proposalSeed",
        "proposalId",
        "recipient",
        "amount",
        "approve",
        "claimedNullifier",
        "witness",
        "execute",
        "fetchConfig",
        "fetchProposal",
        "checkConnection",
        "listAccounts",
        "createAccount",
        "label",
        "inspectAccount",
        "accountId",
        "decodeAccount",
        "fieldHistory",
        "key",
        "saveHistory",
        "value",
        "config",
        "QVariantMap",
        "proposal",
        "busy",
        "lastError",
        "lastTxHash",
        "lastResult",
        "walletPath",
        "sequencerUrl",
        "programIdHex",
        "connectionStatus",
        "walletAccounts",
        "walletAccountInfo",
        "walletDecodedAccount"
    };

    QtMocHelpers::UintData qt_methods {
        // Signal 'configChanged'
        QtMocHelpers::SignalData<void()>(1, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'proposalChanged'
        QtMocHelpers::SignalData<void()>(3, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'busyChanged'
        QtMocHelpers::SignalData<void()>(4, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'lastErrorChanged'
        QtMocHelpers::SignalData<void()>(5, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'lastTxHashChanged'
        QtMocHelpers::SignalData<void()>(6, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'lastResultChanged'
        QtMocHelpers::SignalData<void()>(7, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'operationSuccess'
        QtMocHelpers::SignalData<void(const QString &, const QString &)>(8, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 9 }, { QMetaType::QString, 10 },
        }}),
        // Signal 'operationError'
        QtMocHelpers::SignalData<void(const QString &, const QString &)>(11, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 9 }, { QMetaType::QString, 12 },
        }}),
        // Signal 'walletPathChanged'
        QtMocHelpers::SignalData<void()>(13, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'sequencerUrlChanged'
        QtMocHelpers::SignalData<void()>(14, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'programIdHexChanged'
        QtMocHelpers::SignalData<void()>(15, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'connectionStatusChanged'
        QtMocHelpers::SignalData<void()>(16, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'walletAccountsChanged'
        QtMocHelpers::SignalData<void()>(17, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'walletAccountInfoChanged'
        QtMocHelpers::SignalData<void()>(18, 2, QMC::AccessPublic, QMetaType::Void),
        // Signal 'walletDecodedAccountChanged'
        QtMocHelpers::SignalData<void()>(19, 2, QMC::AccessPublic, QMetaType::Void),
        // Method 'setWalletPath'
        QtMocHelpers::MethodData<void(const QString &)>(20, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 21 },
        }}),
        // Method 'setSequencerUrl'
        QtMocHelpers::MethodData<void(const QString &)>(22, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 21 },
        }}),
        // Method 'setProgramIdHex'
        QtMocHelpers::MethodData<void(const QString &)>(23, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 21 },
        }}),
        // Method 'createMultisig'
        QtMocHelpers::MethodData<void(const QString &, const QString &, const QString &, quint32, quint32, const QString &, const QVariantList &)>(24, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 25 }, { QMetaType::QString, 26 }, { QMetaType::QString, 27 }, { QMetaType::UInt, 28 },
            { QMetaType::UInt, 29 }, { QMetaType::QString, 30 }, { 0x80000000 | 31, 32 },
        }}),
        // Method 'createProposal'
        QtMocHelpers::MethodData<void(const QString &, const QString &, const QString &, const QString &, const QString &, const QString &)>(33, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 34 }, { QMetaType::QString, 26 }, { QMetaType::QString, 35 }, { QMetaType::QString, 36 },
            { QMetaType::QString, 37 }, { QMetaType::QString, 38 },
        }}),
        // Method 'approve'
        QtMocHelpers::MethodData<void(const QString &, const QString &, const QString &, const QString &, const QVariantList &)>(39, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 26 }, { QMetaType::QString, 35 }, { QMetaType::QString, 27 }, { QMetaType::QString, 40 },
            { 0x80000000 | 31, 41 },
        }}),
        // Method 'execute'
        QtMocHelpers::MethodData<void(const QString &, const QString &)>(42, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 26 }, { QMetaType::QString, 35 },
        }}),
        // Method 'fetchConfig'
        QtMocHelpers::MethodData<void(const QString &)>(43, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 26 },
        }}),
        // Method 'fetchProposal'
        QtMocHelpers::MethodData<void(const QString &)>(44, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 35 },
        }}),
        // Method 'checkConnection'
        QtMocHelpers::MethodData<void()>(45, 2, QMC::AccessPublic, QMetaType::Void),
        // Method 'listAccounts'
        QtMocHelpers::MethodData<void()>(46, 2, QMC::AccessPublic, QMetaType::Void),
        // Method 'createAccount'
        QtMocHelpers::MethodData<void(const QString &)>(47, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 48 },
        }}),
        // Method 'inspectAccount'
        QtMocHelpers::MethodData<void(const QString &)>(49, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 50 },
        }}),
        // Method 'decodeAccount'
        QtMocHelpers::MethodData<void(const QString &)>(51, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 50 },
        }}),
        // Method 'fieldHistory'
        QtMocHelpers::MethodData<QStringList(const QString &) const>(52, 2, QMC::AccessPublic, QMetaType::QStringList, {{
            { QMetaType::QString, 53 },
        }}),
        // Method 'saveHistory'
        QtMocHelpers::MethodData<void(const QString &, const QString &)>(54, 2, QMC::AccessPublic, QMetaType::Void, {{
            { QMetaType::QString, 53 }, { QMetaType::QString, 55 },
        }}),
    };
    QtMocHelpers::UintData qt_properties {
        // property 'config'
        QtMocHelpers::PropertyData<QVariantMap>(56, 0x80000000 | 57, QMC::DefaultPropertyFlags | QMC::EnumOrFlag, 0),
        // property 'proposal'
        QtMocHelpers::PropertyData<QVariantMap>(58, 0x80000000 | 57, QMC::DefaultPropertyFlags | QMC::EnumOrFlag, 1),
        // property 'busy'
        QtMocHelpers::PropertyData<bool>(59, QMetaType::Bool, QMC::DefaultPropertyFlags, 2),
        // property 'lastError'
        QtMocHelpers::PropertyData<QString>(60, QMetaType::QString, QMC::DefaultPropertyFlags, 3),
        // property 'lastTxHash'
        QtMocHelpers::PropertyData<QString>(61, QMetaType::QString, QMC::DefaultPropertyFlags, 4),
        // property 'lastResult'
        QtMocHelpers::PropertyData<QVariantMap>(62, 0x80000000 | 57, QMC::DefaultPropertyFlags | QMC::EnumOrFlag, 5),
        // property 'walletPath'
        QtMocHelpers::PropertyData<QString>(63, QMetaType::QString, QMC::DefaultPropertyFlags | QMC::Writable | QMC::StdCppSet, 8),
        // property 'sequencerUrl'
        QtMocHelpers::PropertyData<QString>(64, QMetaType::QString, QMC::DefaultPropertyFlags | QMC::Writable | QMC::StdCppSet, 9),
        // property 'programIdHex'
        QtMocHelpers::PropertyData<QString>(65, QMetaType::QString, QMC::DefaultPropertyFlags | QMC::Writable | QMC::StdCppSet, 10),
        // property 'connectionStatus'
        QtMocHelpers::PropertyData<QString>(66, QMetaType::QString, QMC::DefaultPropertyFlags, 11),
        // property 'walletAccounts'
        QtMocHelpers::PropertyData<QVariantList>(67, 0x80000000 | 31, QMC::DefaultPropertyFlags | QMC::EnumOrFlag, 12),
        // property 'walletAccountInfo'
        QtMocHelpers::PropertyData<QVariantMap>(68, 0x80000000 | 57, QMC::DefaultPropertyFlags | QMC::EnumOrFlag, 13),
        // property 'walletDecodedAccount'
        QtMocHelpers::PropertyData<QVariantMap>(69, 0x80000000 | 57, QMC::DefaultPropertyFlags | QMC::EnumOrFlag, 14),
    };
    QtMocHelpers::UintData qt_enums {
    };
    return QtMocHelpers::metaObjectData<PrivateMultisigBackend, qt_meta_tag_ZN22PrivateMultisigBackendE_t>(QMC::MetaObjectFlag{}, qt_stringData,
            qt_methods, qt_properties, qt_enums);
}
Q_CONSTINIT const QMetaObject PrivateMultisigBackend::staticMetaObject = { {
    QMetaObject::SuperData::link<QObject::staticMetaObject>(),
    qt_staticMetaObjectStaticContent<qt_meta_tag_ZN22PrivateMultisigBackendE_t>.stringdata,
    qt_staticMetaObjectStaticContent<qt_meta_tag_ZN22PrivateMultisigBackendE_t>.data,
    qt_static_metacall,
    nullptr,
    qt_staticMetaObjectRelocatingContent<qt_meta_tag_ZN22PrivateMultisigBackendE_t>.metaTypes,
    nullptr
} };

void PrivateMultisigBackend::qt_static_metacall(QObject *_o, QMetaObject::Call _c, int _id, void **_a)
{
    auto *_t = static_cast<PrivateMultisigBackend *>(_o);
    if (_c == QMetaObject::InvokeMetaMethod) {
        switch (_id) {
        case 0: _t->configChanged(); break;
        case 1: _t->proposalChanged(); break;
        case 2: _t->busyChanged(); break;
        case 3: _t->lastErrorChanged(); break;
        case 4: _t->lastTxHashChanged(); break;
        case 5: _t->lastResultChanged(); break;
        case 6: _t->operationSuccess((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[2]))); break;
        case 7: _t->operationError((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[2]))); break;
        case 8: _t->walletPathChanged(); break;
        case 9: _t->sequencerUrlChanged(); break;
        case 10: _t->programIdHexChanged(); break;
        case 11: _t->connectionStatusChanged(); break;
        case 12: _t->walletAccountsChanged(); break;
        case 13: _t->walletAccountInfoChanged(); break;
        case 14: _t->walletDecodedAccountChanged(); break;
        case 15: _t->setWalletPath((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1]))); break;
        case 16: _t->setSequencerUrl((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1]))); break;
        case 17: _t->setProgramIdHex((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1]))); break;
        case 18: _t->createMultisig((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[2])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[3])),(*reinterpret_cast<std::add_pointer_t<quint32>>(_a[4])),(*reinterpret_cast<std::add_pointer_t<quint32>>(_a[5])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[6])),(*reinterpret_cast<std::add_pointer_t<QVariantList>>(_a[7]))); break;
        case 19: _t->createProposal((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[2])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[3])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[4])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[5])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[6]))); break;
        case 20: _t->approve((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[2])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[3])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[4])),(*reinterpret_cast<std::add_pointer_t<QVariantList>>(_a[5]))); break;
        case 21: _t->execute((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[2]))); break;
        case 22: _t->fetchConfig((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1]))); break;
        case 23: _t->fetchProposal((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1]))); break;
        case 24: _t->checkConnection(); break;
        case 25: _t->listAccounts(); break;
        case 26: _t->createAccount((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1]))); break;
        case 27: _t->inspectAccount((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1]))); break;
        case 28: _t->decodeAccount((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1]))); break;
        case 29: { QStringList _r = _t->fieldHistory((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1])));
            if (_a[0]) *reinterpret_cast<QStringList*>(_a[0]) = std::move(_r); }  break;
        case 30: _t->saveHistory((*reinterpret_cast<std::add_pointer_t<QString>>(_a[1])),(*reinterpret_cast<std::add_pointer_t<QString>>(_a[2]))); break;
        default: ;
        }
    }
    if (_c == QMetaObject::IndexOfMethod) {
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::configChanged, 0))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::proposalChanged, 1))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::busyChanged, 2))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::lastErrorChanged, 3))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::lastTxHashChanged, 4))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::lastResultChanged, 5))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)(const QString & , const QString & )>(_a, &PrivateMultisigBackend::operationSuccess, 6))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)(const QString & , const QString & )>(_a, &PrivateMultisigBackend::operationError, 7))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::walletPathChanged, 8))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::sequencerUrlChanged, 9))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::programIdHexChanged, 10))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::connectionStatusChanged, 11))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::walletAccountsChanged, 12))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::walletAccountInfoChanged, 13))
            return;
        if (QtMocHelpers::indexOfMethod<void (PrivateMultisigBackend::*)()>(_a, &PrivateMultisigBackend::walletDecodedAccountChanged, 14))
            return;
    }
    if (_c == QMetaObject::ReadProperty) {
        void *_v = _a[0];
        switch (_id) {
        case 0: *reinterpret_cast<QVariantMap*>(_v) = _t->config(); break;
        case 1: *reinterpret_cast<QVariantMap*>(_v) = _t->proposal(); break;
        case 2: *reinterpret_cast<bool*>(_v) = _t->busy(); break;
        case 3: *reinterpret_cast<QString*>(_v) = _t->lastError(); break;
        case 4: *reinterpret_cast<QString*>(_v) = _t->lastTxHash(); break;
        case 5: *reinterpret_cast<QVariantMap*>(_v) = _t->lastResult(); break;
        case 6: *reinterpret_cast<QString*>(_v) = _t->walletPath(); break;
        case 7: *reinterpret_cast<QString*>(_v) = _t->sequencerUrl(); break;
        case 8: *reinterpret_cast<QString*>(_v) = _t->programIdHex(); break;
        case 9: *reinterpret_cast<QString*>(_v) = _t->connectionStatus(); break;
        case 10: *reinterpret_cast<QVariantList*>(_v) = _t->walletAccounts(); break;
        case 11: *reinterpret_cast<QVariantMap*>(_v) = _t->walletAccountInfo(); break;
        case 12: *reinterpret_cast<QVariantMap*>(_v) = _t->walletDecodedAccount(); break;
        default: break;
        }
    }
    if (_c == QMetaObject::WriteProperty) {
        void *_v = _a[0];
        switch (_id) {
        case 6: _t->setWalletPath(*reinterpret_cast<QString*>(_v)); break;
        case 7: _t->setSequencerUrl(*reinterpret_cast<QString*>(_v)); break;
        case 8: _t->setProgramIdHex(*reinterpret_cast<QString*>(_v)); break;
        default: break;
        }
    }
}

const QMetaObject *PrivateMultisigBackend::metaObject() const
{
    return QObject::d_ptr->metaObject ? QObject::d_ptr->dynamicMetaObject() : &staticMetaObject;
}

void *PrivateMultisigBackend::qt_metacast(const char *_clname)
{
    if (!_clname) return nullptr;
    if (!strcmp(_clname, qt_staticMetaObjectStaticContent<qt_meta_tag_ZN22PrivateMultisigBackendE_t>.strings))
        return static_cast<void*>(this);
    return QObject::qt_metacast(_clname);
}

int PrivateMultisigBackend::qt_metacall(QMetaObject::Call _c, int _id, void **_a)
{
    _id = QObject::qt_metacall(_c, _id, _a);
    if (_id < 0)
        return _id;
    if (_c == QMetaObject::InvokeMetaMethod) {
        if (_id < 31)
            qt_static_metacall(this, _c, _id, _a);
        _id -= 31;
    }
    if (_c == QMetaObject::RegisterMethodArgumentMetaType) {
        if (_id < 31)
            *reinterpret_cast<QMetaType *>(_a[0]) = QMetaType();
        _id -= 31;
    }
    if (_c == QMetaObject::ReadProperty || _c == QMetaObject::WriteProperty
            || _c == QMetaObject::ResetProperty || _c == QMetaObject::BindableProperty
            || _c == QMetaObject::RegisterPropertyMetaType) {
        qt_static_metacall(this, _c, _id, _a);
        _id -= 13;
    }
    return _id;
}

// SIGNAL 0
void PrivateMultisigBackend::configChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 0, nullptr);
}

// SIGNAL 1
void PrivateMultisigBackend::proposalChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 1, nullptr);
}

// SIGNAL 2
void PrivateMultisigBackend::busyChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 2, nullptr);
}

// SIGNAL 3
void PrivateMultisigBackend::lastErrorChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 3, nullptr);
}

// SIGNAL 4
void PrivateMultisigBackend::lastTxHashChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 4, nullptr);
}

// SIGNAL 5
void PrivateMultisigBackend::lastResultChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 5, nullptr);
}

// SIGNAL 6
void PrivateMultisigBackend::operationSuccess(const QString & _t1, const QString & _t2)
{
    QMetaObject::activate<void>(this, &staticMetaObject, 6, nullptr, _t1, _t2);
}

// SIGNAL 7
void PrivateMultisigBackend::operationError(const QString & _t1, const QString & _t2)
{
    QMetaObject::activate<void>(this, &staticMetaObject, 7, nullptr, _t1, _t2);
}

// SIGNAL 8
void PrivateMultisigBackend::walletPathChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 8, nullptr);
}

// SIGNAL 9
void PrivateMultisigBackend::sequencerUrlChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 9, nullptr);
}

// SIGNAL 10
void PrivateMultisigBackend::programIdHexChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 10, nullptr);
}

// SIGNAL 11
void PrivateMultisigBackend::connectionStatusChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 11, nullptr);
}

// SIGNAL 12
void PrivateMultisigBackend::walletAccountsChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 12, nullptr);
}

// SIGNAL 13
void PrivateMultisigBackend::walletAccountInfoChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 13, nullptr);
}

// SIGNAL 14
void PrivateMultisigBackend::walletDecodedAccountChanged()
{
    QMetaObject::activate(this, &staticMetaObject, 14, nullptr);
}
QT_WARNING_POP
