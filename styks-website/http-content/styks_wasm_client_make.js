let wasm;

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_2.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

function passStringToWasm0(arg, malloc, realloc) {

    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }

    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_5.get(state.dtor)(state.a, state.b)
});

function makeClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        try {
            return f(state.a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_5.get(state.dtor)(state.a, state.b);
                state.a = 0;
                CLOSURE_DTORS.unregister(state);
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_5.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    for (let i = 0; i < array.length; i++) {
        const add = addToExternrefTable0(array[i]);
        getDataViewMemory0().setUint32(ptr + 4 * i, add, true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_2.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_export_2.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
    return result;
}
/**
 * Returns the gas limit for the client for the next calls.
 * @returns {bigint}
 */
export function gas() {
    const ret = wasm.gas();
    return BigInt.asUintN(64, ret);
}

/**
 * Sets the gas limit for the client for the next calls.
 * @param {bigint} gas
 */
export function setGas(gas) {
    wasm.setGas(gas);
}

/**
 * Returns the default payment amount for transactions.
 * @returns {bigint}
 */
export function DEFAULT_PAYMENT_AMOUNT() {
    const ret = wasm.DEFAULT_PAYMENT_AMOUNT();
    return BigInt.asUintN(64, ret);
}

export function run() {
    wasm.run();
}

/**
 * @returns {AccountInfo}
 */
export function getCurrentAccount() {
    const ret = wasm.getCurrentAccount();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return AccountInfo.__wrap(ret[0]);
}

function __wbg_adapter_42(arg0, arg1) {
    wasm._dyn_core__ops__function__Fn_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h1619f4ae3a65ac94(arg0, arg1);
}

function __wbg_adapter_45(arg0, arg1, arg2, arg3) {
    wasm.closure364_externref_shim(arg0, arg1, arg2, arg3);
}

function __wbg_adapter_48(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__h7bbc2ba14b70c980(arg0, arg1);
}

function __wbg_adapter_51(arg0, arg1, arg2) {
    wasm.closure808_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_480(arg0, arg1, arg2, arg3) {
    wasm.closure1283_externref_shim(arg0, arg1, arg2, arg3);
}

/**
 * @enum {20002 | 20001 | 20003 | 20000 | 20004}
 */
export const StyksBlockySupplierErrors = Object.freeze({
    /**
     * The caller is not the new owner.
     */
    CallerNotTheNewOwner: 20002, "20002": "CallerNotTheNewOwner",
    /**
     * The caller is not the owner.
     */
    CallerNotTheOwner: 20001, "20001": "CallerNotTheOwner",
    /**
     * The role is missing.
     */
    MissingRole: 20003, "20003": "MissingRole",
    /**
     * The owner is not set.
     */
    OwnerNotSet: 20000, "20000": "OwnerNotSet",
    /**
     * The role cannot be renounced for another address.
     */
    RoleRenounceForAnotherAddress: 20004, "20004": "RoleRenounceForAnotherAddress",
});
/**
 * @enum {20002 | 20001 | 20003 | 20000 | 20004}
 */
export const StyksMakeSupplierErrors = Object.freeze({
    /**
     * The caller is not the new owner.
     */
    CallerNotTheNewOwner: 20002, "20002": "CallerNotTheNewOwner",
    /**
     * The caller is not the owner.
     */
    CallerNotTheOwner: 20001, "20001": "CallerNotTheOwner",
    /**
     * The role is missing.
     */
    MissingRole: 20003, "20003": "MissingRole",
    /**
     * The owner is not set.
     */
    OwnerNotSet: 20000, "20000": "OwnerNotSet",
    /**
     * The role cannot be renounced for another address.
     */
    RoleRenounceForAnotherAddress: 20004, "20004": "RoleRenounceForAnotherAddress",
});
/**
 * @enum {20002 | 20001 | 20003 | 20000 | 20004}
 */
export const StyksPriceFeedErrors = Object.freeze({
    /**
     * The caller is not the new owner.
     */
    CallerNotTheNewOwner: 20002, "20002": "CallerNotTheNewOwner",
    /**
     * The caller is not the owner.
     */
    CallerNotTheOwner: 20001, "20001": "CallerNotTheOwner",
    /**
     * The role is missing.
     */
    MissingRole: 20003, "20003": "MissingRole",
    /**
     * The owner is not set.
     */
    OwnerNotSet: 20000, "20000": "OwnerNotSet",
    /**
     * The role cannot be renounced for another address.
     */
    RoleRenounceForAnotherAddress: 20004, "20004": "RoleRenounceForAnotherAddress",
});
/**
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6}
 */
export const TransactionStatus = Object.freeze({
    /**
     * The transaction has been signed and successfully deployed to a Casper node.
     */
    SENT: 0, "0": "SENT",
    /**
     * The transaction has been processed by the network. May result in success or failure.
     */
    PROCESSED: 1, "1": "PROCESSED",
    /**
     * The transaction’s time-to-live (TTL) elapsed before execution.
     */
    EXPIRED: 2, "2": "EXPIRED",
    /**
     * The user rejected the signature request.
     */
    CANCELLED: 3, "3": "CANCELLED",
    /**
     * The SDK stopped listening for updates before the transaction was finalized. A custom timeout can be specified (default: 120 seconds).
     */
    TIMEOUT: 4, "4": "TIMEOUT",
    /**
     * An unexpected error occurred while submitting or monitoring the transaction.
     */
    ERROR: 5, "5": "ERROR",
    /**
     * A heartbeat event sent periodically to indicate that the connection is still active.
     */
    PING: 6, "6": "PING",
});
/**
 * @enum {0 | 1 | 2}
 */
export const Verbosity = Object.freeze({
    Low: 0, "0": "Low",
    Medium: 1, "1": "Medium",
    High: 2, "2": "High",
});

const __wbindgen_enum_RequestCache = ["default", "no-store", "reload", "no-cache", "force-cache", "only-if-cached"];

const __wbindgen_enum_RequestCredentials = ["omit", "same-origin", "include"];

const __wbindgen_enum_RequestMode = ["same-origin", "no-cors", "cors", "navigate"];

const AccountInfoFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_accountinfo_free(ptr >>> 0, 1));

export class AccountInfo {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AccountInfo.prototype);
        obj.__wbg_ptr = ptr;
        AccountInfoFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            provider: this.provider,
            providerSupports: this.providerSupports,
            csprName: this.csprName,
            publicKey: this.publicKey,
            connectedAt: this.connectedAt,
            logo: this.logo,
            address: this.address,
            balance: this.balance,
            liquidBalance: this.liquidBalance,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AccountInfoFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_accountinfo_free(ptr, 0);
    }
    /**
     * The provider to which the account belongs to.
     * @returns {string}
     */
    get provider() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_accountinfo_provider(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * An array of supported capabilities in the connected wallet.
     * Possible values: "sign-deploy", "sign-transactionv1", "sign-message".
     * @returns {string[] | undefined}
     */
    get providerSupports() {
        const ret = wasm.__wbg_get_accountinfo_providerSupports(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        }
        return v1;
    }
    /**
     * CSPR.name name
     * @returns {string | undefined}
     */
    get csprName() {
        const ret = wasm.__wbg_get_accountinfo_csprName(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * The account public key in hex format.
     * @returns {string}
     */
    get publicKey() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_accountinfo_publicKey(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Timestamp for the initial connection of the account
     * @returns {bigint}
     */
    get connectedAt() {
        const ret = wasm.__wbg_get_accountinfo_connectedAt(this.__wbg_ptr);
        return ret;
    }
    /**
     * URL to the account avatar/logo.
     * @returns {string | undefined}
     */
    get logo() {
        const ret = wasm.__wbg_get_accountinfo_logo(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * The account address derived from the public key.
     * @returns {Address}
     */
    get address() {
        const ret = wasm.accountinfo_address(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Address.__wrap(ret[0]);
    }
    /**
     * Total balance of the account in CSPR motes (includes liquid +staked balance).
     * @returns {U512}
     */
    get balance() {
        const ret = wasm.accountinfo_balance(this.__wbg_ptr);
        return U512.__wrap(ret);
    }
    /**
     * Liquid balance of the account in CSPR motes (includes liquid +staked balance)
     * @returns {U512}
     */
    get liquidBalance() {
        const ret = wasm.accountinfo_liquidBalance(this.__wbg_ptr);
        return U512.__wrap(ret);
    }
}

const AddressFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_address_free(ptr >>> 0, 1));

export class Address {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Address.prototype);
        obj.__wbg_ptr = ptr;
        AddressFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            value: this.value,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AddressFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_address_free(ptr, 0);
    }
    /**
     * @param {string} address
     */
    constructor(address) {
        const ptr0 = passStringToWasm0(address, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.address_new(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        AddressFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {HTMLInputElement} input
     * @returns {Address}
     */
    static fromHtmlInput(input) {
        const ret = wasm.address_fromHtmlInput(input);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Address.__wrap(ret[0]);
    }
    /**
     * @param {string} input
     * @returns {Address}
     */
    static fromPublicKey(input) {
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.address_fromPublicKey(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Address.__wrap(ret[0]);
    }
    /**
     * @returns {string}
     */
    get value() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.address_value(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}

const BalanceFormatterFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_balanceformatter_free(ptr >>> 0, 1));

export class BalanceFormatter {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(BalanceFormatter.prototype);
        obj.__wbg_ptr = ptr;
        BalanceFormatterFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BalanceFormatterFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_balanceformatter_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    fmt() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.balanceformatter_fmt(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {number} precision
     * @returns {string}
     */
    fmtWithPrecision(precision) {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.balanceformatter_fmtWithPrecision(this.__wbg_ptr, precision);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}

const BytesFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_bytes_free(ptr >>> 0, 1));

export class Bytes {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Bytes.prototype);
        obj.__wbg_ptr = ptr;
        BytesFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BytesFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_bytes_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.bytes_new();
        this.__wbg_ptr = ret >>> 0;
        BytesFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Uint8Array} uint8_array
     * @returns {Bytes}
     */
    static fromUint8Array(uint8_array) {
        const ret = wasm.bytes_fromUint8Array(uint8_array);
        return Bytes.__wrap(ret);
    }
    /**
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.bytes_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get value() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.bytes_value(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}

const ContractInfoFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_contractinfo_free(ptr >>> 0, 1));
/**
 * Information about a specific smart contract.
 *
 * Contains essential metadata for identifying and interacting with a deployed
 * smart contract, including its name and package hash used for addressing.
 */
export class ContractInfo {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(ContractInfo.prototype);
        obj.__wbg_ptr = ptr;
        ContractInfoFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            name: this.name,
            package_hash: this.package_hash,
            address: this.address,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ContractInfoFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_contractinfo_free(ptr, 0);
    }
    /**
     * Human-readable name of the contract
     * @returns {string}
     */
    get name() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_contractinfo_name(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Human-readable name of the contract
     * @param {string} arg0
     */
    set name(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_contractinfo_name(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Unique hash identifier for the contract package
     * @returns {string}
     */
    get package_hash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_contractinfo_package_hash(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Unique hash identifier for the contract package
     * @param {string} arg0
     */
    set package_hash(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_contractinfo_package_hash(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * Gets the contract's address derived from its package hash.
     *
     * Converts the package hash into an Address instance that can be used
     * for contract interactions.
     *
     * # Returns
     * * `Address` - Contract address derived from the package hash
     *
     * # Panics
     * Panics if the package hash is invalid and cannot be converted to an Address.
     * @returns {Address}
     */
    get address() {
        const ret = wasm.contractinfo_address(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Address.__wrap(ret[0]);
    }
}

const ContractsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_contracts_free(ptr >>> 0, 1));
/**
 * Container for multiple smart contract definitions with metadata.
 *
 * This structure holds a collection of contract information along with
 * a timestamp indicating when the data was last updated. It can be
 * serialized/deserialized to/from JSON and is exposed to JavaScript.
 */
export class Contracts {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Contracts.prototype);
        obj.__wbg_ptr = ptr;
        ContractsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ContractsFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_contracts_free(ptr, 0);
    }
    /**
     * Creates a new `Contracts` instance from a JavaScript value.
     *
     * # Arguments
     * * `js` - JavaScript value containing contract data in JSON format
     *
     * # Returns
     * * `Result<Self, JsError>` - New Contracts instance or error if parsing fails
     *
     * # Errors
     * Returns `JsError` if the JavaScript value cannot be deserialized into a Contracts struct.
     * @param {any} js
     */
    constructor(js) {
        const ret = wasm.contracts_new(js);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        ContractsFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Asynchronously loads contract information from a remote JSON file.
     *
     * This method fetches contract data from the specified URL path using the browser's
     * fetch API, parses the JSON response, and creates a new Contracts instance.
     *
     * # Arguments
     * * `path` - URL path to the JSON file containing contract information
     *
     * # Returns
     * * `Result<Self, JsError>` - New Contracts instance or error if loading fails
     *
     * # Errors
     * Returns `JsError` if:
     * - No window object is available (not in browser environment)
     * - Network request fails
     * - Response cannot be parsed as JSON
     * - JSON structure doesn't match expected Contracts format
     * @param {string} path
     * @returns {Promise<Contracts>}
     */
    static fromPath(path) {
        const ptr0 = passStringToWasm0(path, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.contracts_fromPath(ptr0, len0);
        return ret;
    }
    /**
     * Retrieves contract information by name.
     *
     * Searches through the contracts collection for a contract with the specified name
     * and returns its information if found.
     *
     * # Arguments
     * * `name` - Name of the contract to find
     *
     * # Returns
     * * `Result<ContractInfo, JsError>` - Contract information or error if not found
     *
     * # Errors
     * Returns `JsError` if no contract with the specified name exists.
     * @param {string} name
     * @returns {ContractInfo}
     */
    get(name) {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.contracts_get(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ContractInfo.__wrap(ret[0]);
    }
}

const CsprClickCallbacksFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_csprclickcallbacks_free(ptr >>> 0, 1));

export class CsprClickCallbacks {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CsprClickCallbacksFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_csprclickcallbacks_free(ptr, 0);
    }
    /**
     * @param {Function} callback
     */
    static onSignedIn(callback) {
        wasm.csprclickcallbacks_onSignedIn(callback);
    }
    /**
     * @param {Function} callback
     */
    static onSwitchedAccount(callback) {
        wasm.csprclickcallbacks_onSwitchedAccount(callback);
    }
    /**
     * @param {Function} callback
     */
    static onUnsolicitedAccountChange(callback) {
        wasm.csprclickcallbacks_onUnsolicitedAccountChange(callback);
    }
    /**
     * @param {Function} callback
     */
    static onSignedOut(callback) {
        wasm.csprclickcallbacks_onSignedOut(callback);
    }
    /**
     * @param {Function} callback
     */
    static onDisconnected(callback) {
        wasm.csprclickcallbacks_onDisconnected(callback);
    }
    /**
     * @param {Function} callback
     */
    static onTransactionStatusUpdate(callback) {
        wasm.csprclickcallbacks_onTransactionStatusUpdate(callback);
    }
}

const MakeSupplierConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_makesupplierconfig_free(ptr >>> 0, 1));

export class MakeSupplierConfig {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(MakeSupplierConfig.prototype);
        obj.__wbg_ptr = ptr;
        MakeSupplierConfigFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        MakeSupplierConfigFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_makesupplierconfig_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get timestampTolerance() {
        const ret = wasm.__wbg_get_makesupplierconfig_timestampTolerance(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set timestampTolerance(arg0) {
        wasm.__wbg_set_makesupplierconfig_timestampTolerance(this.__wbg_ptr, arg0);
    }
    /**
     * @param {PublicKey} publicKey
     * @param {any[]} feedIds
     * @param {Address} priceFeedAddress
     * @param {bigint} timestampTolerance
     */
    constructor(publicKey, feedIds, priceFeedAddress, timestampTolerance) {
        _assertClass(publicKey, PublicKey);
        var ptr0 = publicKey.__destroy_into_raw();
        const ptr1 = passArrayJsValueToWasm0(feedIds, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        _assertClass(priceFeedAddress, Address);
        var ptr2 = priceFeedAddress.__destroy_into_raw();
        const ret = wasm.makesupplierconfig_new(ptr0, ptr1, len1, ptr2, timestampTolerance);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        MakeSupplierConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.makesupplierconfig_toJson(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {PublicKey} value
     */
    set public_key(value) {
        _assertClass(value, PublicKey);
        var ptr0 = value.__destroy_into_raw();
        wasm.makesupplierconfig_set_public_key(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {PublicKey}
     */
    get public_key() {
        const ret = wasm.makesupplierconfig_public_key(this.__wbg_ptr);
        return PublicKey.__wrap(ret);
    }
    /**
     * @param {any[]} value
     */
    set feed_ids(value) {
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.makesupplierconfig_set_feed_ids(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {any[]}
     */
    get feed_ids() {
        const ret = wasm.makesupplierconfig_feed_ids(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {Address} value
     */
    set price_feed_address(value) {
        _assertClass(value, Address);
        var ptr0 = value.__destroy_into_raw();
        wasm.makesupplierconfig_set_price_feed_address(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Address}
     */
    get price_feed_address() {
        const ret = wasm.makesupplierconfig_price_feed_address(this.__wbg_ptr);
        return Address.__wrap(ret);
    }
}

const OdraWasmClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_odrawasmclient_free(ptr >>> 0, 1));
/**
 * A client for interacting with the Casper blockchain and CSPR.click.
 *
 * The `OdraWasmClient` struct provides methods to interact with the Casper blockchain,
 * including querying balances, transferring tokens, and calling smart contract entry points.
 * It also integrates with CSPR.click for account management and transaction signing.
 */
export class OdraWasmClient {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OdraWasmClientFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_odrawasmclient_free(ptr, 0);
    }
    /**
     * @param {string} node_address
     * @param {string} speculative_node_address
     * @param {string | null} [chain_name]
     * @param {number | null} [ttl]
     * @param {Verbosity | null} [verbosity]
     */
    constructor(node_address, speculative_node_address, chain_name, ttl, verbosity) {
        const ptr0 = passStringToWasm0(node_address, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(speculative_node_address, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(chain_name) ? 0 : passStringToWasm0(chain_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len2 = WASM_VECTOR_LEN;
        const ret = wasm.odrawasmclient_new(ptr0, len0, ptr1, len1, ptr2, len2, isLikeNone(ttl) ? 0x100000001 : (ttl) >>> 0, isLikeNone(verbosity) ? 3 : verbosity);
        this.__wbg_ptr = ret >>> 0;
        OdraWasmClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Returns the balance of the specified address.
     * @param {Address} address
     * @returns {Promise<U512>}
     */
    getBalance(address) {
        _assertClass(address, Address);
        const ret = wasm.odrawasmclient_getBalance(this.__wbg_ptr, address.__wbg_ptr);
        return ret;
    }
    /**
     * Returns the balance of the specified address.
     * @returns {Promise<Address>}
     */
    caller() {
        const ret = wasm.odrawasmclient_caller(this.__wbg_ptr);
        return ret;
    }
    /**
     * Transfers the specified amount to the given address.
     * @param {Address} to
     * @param {U512} amount
     * @returns {Promise<TransactionResult>}
     */
    transfer(to, amount) {
        _assertClass(to, Address);
        _assertClass(amount, U512);
        const ret = wasm.odrawasmclient_transfer(this.__wbg_ptr, to.__wbg_ptr, amount.__wbg_ptr);
        return ret;
    }
    /**
     * Call the connect() method using a provider name as the first parameter to request a connection using that wallet
     * or login mechanism.
     *
     * Some providers may need an options argument to indicate the connection behavior requested.
     * @param {string} provider
     * @returns {Promise<AccountInfo>}
     */
    connect(provider) {
        const ptr0 = passStringToWasm0(provider, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.odrawasmclient_connect(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Triggers a request to a UI library to show a sign-in dialog.
     * @returns {Promise<void>}
     */
    signIn() {
        const ret = wasm.odrawasmclient_signIn(this.__wbg_ptr);
        return ret;
    }
    /**
     * Closes an active session in your dApp.
     *
     * Triggers the [Event::SignedOut](crate::cspr_click::event::Event::SignedOut) event.
     * @returns {Promise<void>}
     */
    signOut() {
        const ret = wasm.odrawasmclient_signOut(this.__wbg_ptr);
        return ret;
    }
    /**
     * Usually you will call signOut() method to close a user session. Use disconnect() when you want to clear
     * the connection between the wallet and your app. Next time the user signs in with that wallet, he'll
     * must grant connection permission again.
     * @returns {Promise<boolean>}
     */
    disconnect() {
        const ret = wasm.odrawasmclient_disconnect(this.__wbg_ptr);
        return ret;
    }
    /**
     * Starts a session with the indicated account. This account must be one of the accounts returned
     * in getKnownAccounts or getSignInOptions.
     *
     * Note that no interaction with the account provider is required to sign-in. CSPR.click will check and restore
     * the connection if needed when there's a transaction or message to sign.
     * @param {AccountInfo} account
     * @returns {Promise<AccountInfo>}
     */
    signInWithAccount(account) {
        _assertClass(account, AccountInfo);
        const ret = wasm.odrawasmclient_signInWithAccount(this.__wbg_ptr, account.__wbg_ptr);
        return ret;
    }
    /**
     * Returns true if the provider is unlocked. false if the provider is locked.
     * @param {string} provider
     * @returns {Promise<boolean>}
     */
    isUnlocked(provider) {
        const ptr0 = passStringToWasm0(provider, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.odrawasmclient_isUnlocked(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Gets the public key for the current session (if any).
     * @returns {Promise<string>}
     */
    getActivePublicKey() {
        const ret = wasm.odrawasmclient_getActivePublicKey(this.__wbg_ptr);
        return ret;
    }
    /**
     * Gets the account for the current session (if any).
     * @returns {Promise<AccountInfo>}
     */
    getActiveAccount() {
        const ret = wasm.odrawasmclient_getActiveAccount(this.__wbg_ptr);
        return ret;
    }
    /**
     * Call this method to request CSPR.click UI to show the Switch Account modal window.
     * @returns {Promise<void>}
     */
    switchAccount() {
        const ret = wasm.odrawasmclient_switchAccount(this.__wbg_ptr);
        return ret;
    }
    /**
     * Triggers the mechanisms to request your user to sign a text message with the active wallet.
     * @param {string} message
     * @returns {Promise<SignResult>}
     */
    signMessage(message) {
        const ptr0 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.odrawasmclient_signMessage(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
}

const OverflowingResultU128Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_overflowingresultu128_free(ptr >>> 0, 1));

export class OverflowingResultU128 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(OverflowingResultU128.prototype);
        obj.__wbg_ptr = ptr;
        OverflowingResultU128Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OverflowingResultU128Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_overflowingresultu128_free(ptr, 0);
    }
    /**
     * @returns {U128}
     */
    get result() {
        const ret = wasm.__wbg_get_overflowingresultu128_result(this.__wbg_ptr);
        return U128.__wrap(ret);
    }
    /**
     * @returns {boolean}
     */
    get overflow() {
        const ret = wasm.__wbg_get_overflowingresultu128_overflow(this.__wbg_ptr);
        return ret !== 0;
    }
}

const OverflowingResultU256Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_overflowingresultu256_free(ptr >>> 0, 1));

export class OverflowingResultU256 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(OverflowingResultU256.prototype);
        obj.__wbg_ptr = ptr;
        OverflowingResultU256Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OverflowingResultU256Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_overflowingresultu256_free(ptr, 0);
    }
    /**
     * @returns {U256}
     */
    get result() {
        const ret = wasm.__wbg_get_overflowingresultu256_result(this.__wbg_ptr);
        return U256.__wrap(ret);
    }
    /**
     * @returns {boolean}
     */
    get overflow() {
        const ret = wasm.__wbg_get_overflowingresultu256_overflow(this.__wbg_ptr);
        return ret !== 0;
    }
}

const OverflowingResultU512Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_overflowingresultu512_free(ptr >>> 0, 1));

export class OverflowingResultU512 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(OverflowingResultU512.prototype);
        obj.__wbg_ptr = ptr;
        OverflowingResultU512Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        OverflowingResultU512Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_overflowingresultu512_free(ptr, 0);
    }
    /**
     * @returns {U512}
     */
    get result() {
        const ret = wasm.__wbg_get_overflowingresultu512_result(this.__wbg_ptr);
        return U512.__wrap(ret);
    }
    /**
     * @returns {boolean}
     */
    get overflow() {
        const ret = wasm.__wbg_get_overflowingresultu512_overflow(this.__wbg_ptr);
        return ret !== 0;
    }
}

const PublicKeyFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_publickey_free(ptr >>> 0, 1));

export class PublicKey {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(PublicKey.prototype);
        obj.__wbg_ptr = ptr;
        PublicKeyFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PublicKeyFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_publickey_free(ptr, 0);
    }
    /**
     * @param {string} public_key_hex_str
     */
    constructor(public_key_hex_str) {
        const ptr0 = passStringToWasm0(public_key_hex_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.publickey_new_js_alias(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        PublicKeyFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Uint8Array} bytes
     * @returns {PublicKey}
     */
    static fromUint8Array(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.publickey_fromUint8Array(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return PublicKey.__wrap(ret[0]);
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.publickey_toJson(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}

const RoleAdminChangedFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_roleadminchanged_free(ptr >>> 0, 1));

export class RoleAdminChanged {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        RoleAdminChangedFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_roleadminchanged_free(ptr, 0);
    }
    /**
     * @returns {Uint8Array}
     */
    get role() {
        const ret = wasm.__wbg_get_roleadminchanged_role(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {Uint8Array} arg0
     */
    set role(arg0) {
        const ptr0 = passArray8ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_roleadminchanged_role(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Uint8Array}
     */
    get previousAdminRole() {
        const ret = wasm.__wbg_get_roleadminchanged_previousAdminRole(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {Uint8Array} arg0
     */
    set previousAdminRole(arg0) {
        const ptr0 = passArray8ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_roleadminchanged_previousAdminRole(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Uint8Array}
     */
    get newAdminRole() {
        const ret = wasm.__wbg_get_roleadminchanged_newAdminRole(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {Uint8Array} arg0
     */
    set newAdminRole(arg0) {
        const ptr0 = passArray8ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_roleadminchanged_newAdminRole(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {Uint8Array} role
     * @param {Uint8Array} previousAdminRole
     * @param {Uint8Array} newAdminRole
     */
    constructor(role, previousAdminRole, newAdminRole) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(previousAdminRole, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArray8ToWasm0(newAdminRole, wasm.__wbindgen_malloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.roleadminchanged_new(ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        RoleAdminChangedFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.roleadminchanged_toJson(this.__wbg_ptr);
        return ret;
    }
}

const RoleGrantedFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_rolegranted_free(ptr >>> 0, 1));

export class RoleGranted {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        RoleGrantedFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_rolegranted_free(ptr, 0);
    }
    /**
     * @returns {Uint8Array}
     */
    get role() {
        const ret = wasm.__wbg_get_rolegranted_role(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {Uint8Array} arg0
     */
    set role(arg0) {
        const ptr0 = passArray8ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_roleadminchanged_role(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {Uint8Array} role
     * @param {Address} address
     * @param {Address} sender
     */
    constructor(role, address, sender) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        _assertClass(sender, Address);
        var ptr2 = sender.__destroy_into_raw();
        const ret = wasm.rolegranted_new(ptr0, len0, ptr1, ptr2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        RoleGrantedFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.rolegranted_toJson(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {Address} value
     */
    set address(value) {
        _assertClass(value, Address);
        var ptr0 = value.__destroy_into_raw();
        wasm.rolegranted_set_address(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Address}
     */
    get address() {
        const ret = wasm.rolegranted_address(this.__wbg_ptr);
        return Address.__wrap(ret);
    }
    /**
     * @param {Address} value
     */
    set sender(value) {
        _assertClass(value, Address);
        var ptr0 = value.__destroy_into_raw();
        wasm.rolegranted_set_sender(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Address}
     */
    get sender() {
        const ret = wasm.rolegranted_sender(this.__wbg_ptr);
        return Address.__wrap(ret);
    }
}

const RoleRevokedFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_rolerevoked_free(ptr >>> 0, 1));

export class RoleRevoked {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        RoleRevokedFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_rolerevoked_free(ptr, 0);
    }
    /**
     * @returns {Uint8Array}
     */
    get role() {
        const ret = wasm.__wbg_get_rolerevoked_role(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {Uint8Array} arg0
     */
    set role(arg0) {
        const ptr0 = passArray8ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_roleadminchanged_role(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {Uint8Array} role
     * @param {Address} address
     * @param {Address} sender
     */
    constructor(role, address, sender) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        _assertClass(sender, Address);
        var ptr2 = sender.__destroy_into_raw();
        const ret = wasm.rolerevoked_new(ptr0, len0, ptr1, ptr2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        RoleRevokedFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.rolerevoked_toJson(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {Address} value
     */
    set address(value) {
        _assertClass(value, Address);
        var ptr0 = value.__destroy_into_raw();
        wasm.rolerevoked_set_address(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Address}
     */
    get address() {
        const ret = wasm.rolegranted_address(this.__wbg_ptr);
        return Address.__wrap(ret);
    }
    /**
     * @param {Address} value
     */
    set sender(value) {
        _assertClass(value, Address);
        var ptr0 = value.__destroy_into_raw();
        wasm.rolerevoked_set_sender(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Address}
     */
    get sender() {
        const ret = wasm.rolegranted_sender(this.__wbg_ptr);
        return Address.__wrap(ret);
    }
}

const SignResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_signresult_free(ptr >>> 0, 1));

export class SignResult {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(SignResult.prototype);
        obj.__wbg_ptr = ptr;
        SignResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SignResultFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_signresult_free(ptr, 0);
    }
    /**
     * true when the user has declined the signature of the transaction. false otherwise
     * @returns {boolean}
     */
    get isCancelled() {
        const ret = wasm.__wbg_get_signresult_isCancelled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * An hexadecimal string with the crytpographic signature of the deploy.
     * @returns {string | undefined}
     */
    get signatureHex() {
        const ret = wasm.__wbg_get_signresult_signatureHex(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * A byte array with the cryptographic signature of the deploy.
     * @returns {Uint8Array}
     */
    get signature() {
        const ret = wasm.__wbg_get_signresult_signature(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * None if the deploy has been successfully signed. It contains an error message otherwise.
     * @returns {string | undefined}
     */
    get error() {
        const ret = wasm.__wbg_get_signresult_error(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {any}
     */
    get transaction() {
        const ret = wasm.signresult_transaction(this.__wbg_ptr);
        return ret;
    }
}

const StyksBlockySupplerConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_styksblockysupplerconfig_free(ptr >>> 0, 1));

export class StyksBlockySupplerConfig {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StyksBlockySupplerConfig.prototype);
        obj.__wbg_ptr = ptr;
        StyksBlockySupplerConfigFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StyksBlockySupplerConfigFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_styksblockysupplerconfig_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get wasmHash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_styksblockysupplerconfig_wasmHash(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @param {string} arg0
     */
    set wasmHash(arg0) {
        const ptr0 = passStringToWasm0(arg0, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_styksblockysupplerconfig_wasmHash(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Uint8Array}
     */
    get publicKey() {
        const ret = wasm.__wbg_get_styksblockysupplerconfig_publicKey(this.__wbg_ptr);
        var v1 = getArrayU8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v1;
    }
    /**
     * @param {Uint8Array} arg0
     */
    set publicKey(arg0) {
        const ptr0 = passArray8ToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_roleadminchanged_newAdminRole(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {bigint}
     */
    get timestampTolerance() {
        const ret = wasm.__wbg_get_makesupplierconfig_timestampTolerance(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set timestampTolerance(arg0) {
        wasm.__wbg_set_makesupplierconfig_timestampTolerance(this.__wbg_ptr, arg0);
    }
    /**
     * @param {string} wasmHash
     * @param {Uint8Array} publicKey
     * @param {any[]} coingeckoFeedIds
     * @param {Address} priceFeedAddress
     * @param {bigint} timestampTolerance
     */
    constructor(wasmHash, publicKey, coingeckoFeedIds, priceFeedAddress, timestampTolerance) {
        const ptr0 = passStringToWasm0(wasmHash, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(publicKey, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArrayJsValueToWasm0(coingeckoFeedIds, wasm.__wbindgen_malloc);
        const len2 = WASM_VECTOR_LEN;
        _assertClass(priceFeedAddress, Address);
        var ptr3 = priceFeedAddress.__destroy_into_raw();
        const ret = wasm.styksblockysupplerconfig_new(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, timestampTolerance);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        StyksBlockySupplerConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.styksblockysupplerconfig_toJson(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any[]} value
     */
    set coingecko_feed_ids(value) {
        const ptr0 = passArrayJsValueToWasm0(value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.styksblockysupplerconfig_set_coingecko_feed_ids(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {any[]}
     */
    get coingecko_feed_ids() {
        const ret = wasm.styksblockysupplerconfig_coingecko_feed_ids(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {Address} value
     */
    set price_feed_address(value) {
        _assertClass(value, Address);
        var ptr0 = value.__destroy_into_raw();
        wasm.styksblockysupplerconfig_set_price_feed_address(this.__wbg_ptr, ptr0);
    }
    /**
     * @returns {Address}
     */
    get price_feed_address() {
        const ret = wasm.styksblockysupplerconfig_price_feed_address(this.__wbg_ptr);
        return Address.__wrap(ret);
    }
}

const StyksBlockySupplierWasmClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_styksblockysupplierwasmclient_free(ptr >>> 0, 1));

export class StyksBlockySupplierWasmClient {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StyksBlockySupplierWasmClientFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_styksblockysupplierwasmclient_free(ptr, 0);
    }
    /**
     * @param {OdraWasmClient} wasmClient
     * @param {Address} address
     */
    constructor(wasmClient, address) {
        _assertClass(wasmClient, OdraWasmClient);
        _assertClass(address, Address);
        var ptr0 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_new(wasmClient.__wbg_ptr, ptr0);
        this.__wbg_ptr = ret >>> 0;
        StyksBlockySupplierWasmClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {StyksBlockySupplerConfig} config
     * @returns {Promise<TransactionResult>}
     */
    setConfig(config) {
        _assertClass(config, StyksBlockySupplerConfig);
        var ptr0 = config.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_setConfig(this.__wbg_ptr, ptr0);
        return ret;
    }
    /**
     * @returns {Promise<StyksBlockySupplerConfig>}
     */
    getConfig() {
        const ret = wasm.styksblockysupplierwasmclient_getConfig(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Promise<StyksBlockySupplerConfig | undefined>}
     */
    getConfigOrNone() {
        const ret = wasm.styksblockysupplierwasmclient_getConfigOrNone(this.__wbg_ptr);
        return ret;
    }
    /**
     * Verifies the signature against the data.
     * @param {Uint8Array} signature
     * @param {Uint8Array} data
     * @returns {Promise<TransactionResult>}
     */
    reportSignedPrices(signature, data) {
        const ptr0 = passArray8ToWasm0(signature, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.styksblockysupplierwasmclient_reportSignedPrices(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.has_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<boolean>}
     */
    hasRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_hasRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.grant_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    grantRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_grantRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.revoke_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    revokeRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_revokeRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.get_role_admin()` for details.
     * @param {Uint8Array} role
     * @returns {Promise<Uint8Array>}
     */
    getRoleAdmin(role) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.styksblockysupplierwasmclient_getRoleAdmin(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.renounce_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    renounceRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_renounceRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
}

const StyksMakeSupplierWasmClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_styksmakesupplierwasmclient_free(ptr >>> 0, 1));

export class StyksMakeSupplierWasmClient {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StyksMakeSupplierWasmClientFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_styksmakesupplierwasmclient_free(ptr, 0);
    }
    /**
     * @param {OdraWasmClient} wasmClient
     * @param {Address} address
     */
    constructor(wasmClient, address) {
        _assertClass(wasmClient, OdraWasmClient);
        _assertClass(address, Address);
        var ptr0 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_new(wasmClient.__wbg_ptr, ptr0);
        this.__wbg_ptr = ret >>> 0;
        StyksMakeSupplierWasmClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {PublicKey} publicKey
     * @returns {Promise<TransactionResult>}
     */
    updatePublicKey(publicKey) {
        _assertClass(publicKey, PublicKey);
        var ptr0 = publicKey.__destroy_into_raw();
        const ret = wasm.styksmakesupplierwasmclient_updatePublicKey(this.__wbg_ptr, ptr0);
        return ret;
    }
    /**
     * @param {MakeSupplierConfig} config
     * @returns {Promise<TransactionResult>}
     */
    setConfig(config) {
        _assertClass(config, MakeSupplierConfig);
        var ptr0 = config.__destroy_into_raw();
        const ret = wasm.styksmakesupplierwasmclient_setConfig(this.__wbg_ptr, ptr0);
        return ret;
    }
    /**
     * @returns {Promise<MakeSupplierConfig>}
     */
    getConfig() {
        const ret = wasm.styksmakesupplierwasmclient_getConfig(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Promise<MakeSupplierConfig | undefined>}
     */
    getConfigOrNone() {
        const ret = wasm.styksmakesupplierwasmclient_getConfigOrNone(this.__wbg_ptr);
        return ret;
    }
    /**
     * Verifies the signature against the data.
     * @param {Uint8Array} signature
     * @param {Uint8Array} data
     * @returns {Promise<TransactionResult>}
     */
    reportSignedPrices(signature, data) {
        const ptr0 = passArray8ToWasm0(signature, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.styksmakesupplierwasmclient_reportSignedPrices(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.has_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<boolean>}
     */
    hasRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksmakesupplierwasmclient_hasRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.grant_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    grantRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksmakesupplierwasmclient_grantRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.revoke_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    revokeRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksmakesupplierwasmclient_revokeRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.get_role_admin()` for details.
     * @param {Uint8Array} role
     * @returns {Promise<Uint8Array>}
     */
    getRoleAdmin(role) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.styksmakesupplierwasmclient_getRoleAdmin(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.renounce_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    renounceRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksmakesupplierwasmclient_renounceRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
}

const StyksPriceFeedConfigFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stykspricefeedconfig_free(ptr >>> 0, 1));

export class StyksPriceFeedConfig {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(StyksPriceFeedConfig.prototype);
        obj.__wbg_ptr = ptr;
        StyksPriceFeedConfigFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StyksPriceFeedConfigFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stykspricefeedconfig_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get heartbeatInterval() {
        const ret = wasm.__wbg_get_makesupplierconfig_timestampTolerance(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set heartbeatInterval(arg0) {
        wasm.__wbg_set_makesupplierconfig_timestampTolerance(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {bigint}
     */
    get heartbeatTolerance() {
        const ret = wasm.__wbg_get_stykspricefeedconfig_heartbeatTolerance(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set heartbeatTolerance(arg0) {
        wasm.__wbg_set_stykspricefeedconfig_heartbeatTolerance(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get twapWindow() {
        const ret = wasm.__wbg_get_stykspricefeedconfig_twapWindow(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set twapWindow(arg0) {
        wasm.__wbg_set_stykspricefeedconfig_twapWindow(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {number}
     */
    get twapTolerance() {
        const ret = wasm.__wbg_get_stykspricefeedconfig_twapTolerance(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @param {number} arg0
     */
    set twapTolerance(arg0) {
        wasm.__wbg_set_stykspricefeedconfig_twapTolerance(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {string[]}
     */
    get priceFeedIds() {
        const ret = wasm.__wbg_get_stykspricefeedconfig_priceFeedIds(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {string[]} arg0
     */
    set priceFeedIds(arg0) {
        const ptr0 = passArrayJsValueToWasm0(arg0, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.__wbg_set_stykspricefeedconfig_priceFeedIds(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @param {bigint} heartbeatInterval
     * @param {bigint} heartbeatTolerance
     * @param {number} twapWindow
     * @param {number} twapTolerance
     * @param {string[]} priceFeedIds
     */
    constructor(heartbeatInterval, heartbeatTolerance, twapWindow, twapTolerance, priceFeedIds) {
        const ptr0 = passArrayJsValueToWasm0(priceFeedIds, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stykspricefeedconfig_new(heartbeatInterval, heartbeatTolerance, twapWindow, twapTolerance, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        StyksPriceFeedConfigFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.stykspricefeedconfig_toJson(this.__wbg_ptr);
        return ret;
    }
}

const StyksPriceFeedWasmClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_stykspricefeedwasmclient_free(ptr >>> 0, 1));

export class StyksPriceFeedWasmClient {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        StyksPriceFeedWasmClientFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_stykspricefeedwasmclient_free(ptr, 0);
    }
    /**
     * @param {OdraWasmClient} wasmClient
     * @param {Address} address
     */
    constructor(wasmClient, address) {
        _assertClass(wasmClient, OdraWasmClient);
        _assertClass(address, Address);
        var ptr0 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_new(wasmClient.__wbg_ptr, ptr0);
        this.__wbg_ptr = ret >>> 0;
        StyksPriceFeedWasmClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {StyksPriceFeedConfig} config
     * @returns {Promise<TransactionResult>}
     */
    setConfig(config) {
        _assertClass(config, StyksPriceFeedConfig);
        var ptr0 = config.__destroy_into_raw();
        const ret = wasm.stykspricefeedwasmclient_setConfig(this.__wbg_ptr, ptr0);
        return ret;
    }
    /**
     * @returns {Promise<StyksPriceFeedConfig>}
     */
    getConfig() {
        const ret = wasm.stykspricefeedwasmclient_getConfig(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Promise<StyksPriceFeedConfig | undefined>}
     */
    getConfigOrNone() {
        const ret = wasm.stykspricefeedwasmclient_getConfigOrNone(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {string} id
     * @returns {Promise<any[]>}
     */
    getCurrentTwapStore(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stykspricefeedwasmclient_getCurrentTwapStore(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * @returns {Promise<bigint | undefined>}
     */
    getLastHeartbeat() {
        const ret = wasm.stykspricefeedwasmclient_getLastHeartbeat(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {any[]} input
     * @returns {Promise<TransactionResult>}
     */
    addToFeed(input) {
        const ptr0 = passArrayJsValueToWasm0(input, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stykspricefeedwasmclient_addToFeed(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * @param {string} id
     * @returns {Promise<bigint | undefined>}
     */
    getTwapPrice(id) {
        const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stykspricefeedwasmclient_getTwapPrice(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.has_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<boolean>}
     */
    hasRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.stykspricefeedwasmclient_hasRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.grant_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    grantRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.stykspricefeedwasmclient_grantRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.revoke_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    revokeRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.stykspricefeedwasmclient_revokeRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.get_role_admin()` for details.
     * @param {Uint8Array} role
     * @returns {Promise<Uint8Array>}
     */
    getRoleAdmin(role) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.stykspricefeedwasmclient_getRoleAdmin(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
    /**
     * Delegated. See `self.access_control.renounce_role()` for details.
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionResult>}
     */
    renounceRole(role, address) {
        const ptr0 = passArray8ToWasm0(role, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.stykspricefeedwasmclient_renounceRole(this.__wbg_ptr, ptr0, len0, ptr1);
        return ret;
    }
}

const TransactionDataFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_transactiondata_free(ptr >>> 0, 1));

export class TransactionData {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TransactionData.prototype);
        obj.__wbg_ptr = ptr;
        TransactionDataFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TransactionDataFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_transactiondata_free(ptr, 0);
    }
    /**
     * Hash of the block containing the transaction represented as a hexadecimal string.
     * @returns {string}
     */
    get blockHash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_blockHash(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Height of the block containing the transaction.
     * @returns {bigint}
     */
    get blockHeight() {
        const ret = wasm.__wbg_get_transactiondata_blockHeight(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Hash of the transaction caller account represented as a hexademical string.
     * @returns {string}
     */
    get callerHash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_callerHash(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Public key of the transaction caller account represented as a hexademical string.
     * May be null if the public key is not known, but the callerHash will still be present.
     * @returns {string | undefined}
     */
    get callerPublicKey() {
        const ret = wasm.__wbg_get_transactiondata_callerPublicKey(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Represents the total amount of gas consumed during the execution of the transaction.
     * @returns {string}
     */
    get consumedGas() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_consumedGas(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Hash of the contract called by the transaction represented as a hexadecimal string.
     * null if the transaction had no contract call.
     * @returns {string}
     */
    get contractHash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_contractHash(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Hash of the contract package called by the transaction represented as a hexadecimal string.
     * null if the transaction had no contract call.
     * @returns {string}
     */
    get contractPackageHash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_contractPackageHash(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Transaction execution cost. The type is string to avoid overflow in languages that don't support uint64,
     * which is the correct type.
     * @returns {string}
     */
    get cost() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_cost(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Transaction hash represented as a hexadecimal string. Primary transaction identifier.
     * @returns {string}
     */
    get transactionHash() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_transactionHash(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Identifier of the ContractEntrypoint called by transaction. null if the transaction had no contract call.
     * @returns {bigint}
     */
    get entryPointId() {
        const ret = wasm.__wbg_get_transactiondata_entryPointId(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Error message in case of a failed transaction. null for a successful transaction.
     * @returns {string | undefined}
     */
    get errorMessage() {
        const ret = wasm.__wbg_get_transactiondata_errorMessage(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Identifier, that tells what type of the transaction was executed
     * @returns {bigint}
     */
    get executionTypeId() {
        const ret = wasm.__wbg_get_transactiondata_executionTypeId(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Maximum allowed gas price that was specified by the caller (used only in Limited(0) pricing mode)
     * @returns {bigint}
     */
    get gasPriceLimit() {
        const ret = wasm.__wbg_get_transactiondata_gasPriceLimit(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Indicates whether the transaction uses the standard payment mechanism or a custom payment contract
     * @returns {boolean}
     */
    get isStandardPayment() {
        const ret = wasm.__wbg_get_transactiondata_isStandardPayment(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Payment amount provided by the caller in motes. The type is string to avoid overflow in languages
     * that don't support uint64, which is the correct type. null if a custom payment contract was provided
     * to the transaction instead of the value in motes.
     * @returns {string}
     */
    get paymentAmount() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_paymentAmount(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Pricing mode identifier. Indicates which pricing model applies to the transaction
     * @returns {bigint}
     */
    get pricingModeId() {
        const ret = wasm.__wbg_get_transactiondata_pricingModeId(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * The amount of gas cost that was refunded to the caller account. In the current Mainnet configuration,
     * 75% of unused payment amount is refunded.
     * @returns {string}
     */
    get refundAmount() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_refundAmount(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Identifies how the transaction was executed: 0 for native execution, 1 for VM version 1, and 2 for VM version 2
     * @returns {bigint}
     */
    get runtimeTypeId() {
        const ret = wasm.__wbg_get_transactiondata_runtimeTypeId(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Transaction status (pending, expired, or processed).
     * @returns {TransactionStatus}
     */
    get status() {
        const ret = wasm.__wbg_get_transactiondata_status(this.__wbg_ptr);
        return ret;
    }
    /**
     * Transaction creation timestamp in the ISO 8601 format.
     * @returns {string}
     */
    get timestamp() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.__wbg_get_transactiondata_timestamp(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Transaction version identifier: 0 for Casper 1.X transactions, 1 for Casper 2.0 transactions, and 2 for Casper 2.0 transactions.
     * @returns {bigint}
     */
    get versionId() {
        const ret = wasm.__wbg_get_transactiondata_versionId(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @returns {any}
     */
    get args() {
        const ret = wasm.transactiondata_args(this.__wbg_ptr);
        return ret;
    }
}

const TransactionResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_transactionresult_free(ptr >>> 0, 1));

export class TransactionResult {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TransactionResult.prototype);
        obj.__wbg_ptr = ptr;
        TransactionResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TransactionResultFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_transactionresult_free(ptr, 0);
    }
    /**
     * Transaction status (pending, expired, or processed).
     * @returns {TransactionStatus | undefined}
     */
    get status() {
        const ret = wasm.__wbg_get_transactionresult_status(this.__wbg_ptr);
        return ret === 7 ? undefined : ret;
    }
    /**
     * true when the user has declined the signature of the transaction. false otherwise
     * @returns {boolean}
     */
    get isCancelled() {
        const ret = wasm.__wbg_get_transactionresult_isCancelled(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * None if the deploy has been successfully executed. It contains an error message otherwise.
     * @returns {string | undefined}
     */
    get error() {
        const ret = wasm.__wbg_get_transactionresult_error(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * Error code extracted from the error message, if available.
     * @returns {number | undefined}
     */
    get errorCode() {
        const ret = wasm.__wbg_get_transactionresult_errorCode(this.__wbg_ptr);
        return ret === 0xFFFFFF ? undefined : ret;
    }
    /**
     * Transaction details from CSPR Cloud API. null if the transaction is not found or not processed yet.
     * @returns {TransactionData | undefined}
     */
    get data() {
        const ret = wasm.__wbg_get_transactionresult_data(this.__wbg_ptr);
        return ret === 0 ? undefined : TransactionData.__wrap(ret);
    }
    /**
     * If transactionHash is null, returns deployHash. Otherwise, returns transactionHash.
     * @returns {string | undefined}
     */
    get txHash() {
        const ret = wasm.transactionresult_txHash(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
}

const U128Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u128_free(ptr >>> 0, 1));

export class U128 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(U128.prototype);
        obj.__wbg_ptr = ptr;
        U128Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            value: this.value,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U128Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u128_free(ptr, 0);
    }
    /**
     * @param {string} value
     */
    constructor(value) {
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.u128_from_dec_str(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        U128Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} value
     * @returns {U128}
     */
    static fromNumber(value) {
        const ret = wasm.u128_fromNumber(value);
        return U128.__wrap(ret);
    }
    /**
     * @param {HTMLInputElement} input
     * @returns {U128}
     */
    static fromHtmlInput(input) {
        const ret = wasm.u128_fromHtmlInput(input);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U128.__wrap(ret[0]);
    }
    /**
     * @param {bigint} value
     * @returns {U128}
     */
    static fromBigInt(value) {
        const ret = wasm.u128_fromBigInt(value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U128.__wrap(ret[0]);
    }
    /**
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.u128_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.u128_toJson(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} decimals
     * @returns {BalanceFormatter}
     */
    formatter(decimals) {
        const ret = wasm.u128_formatter(this.__wbg_ptr, decimals);
        return BalanceFormatter.__wrap(ret);
    }
    /**
     * @param {U128} other
     * @returns {U128}
     */
    mul(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_mul(this.__wbg_ptr, other.__wbg_ptr);
        return U128.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U128}
     */
    mulBigInt(other) {
        const ret = wasm.u128_mulBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U128.__wrap(ret[0]);
    }
    /**
     * @param {U128} other
     * @returns {U128}
     */
    div(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_div(this.__wbg_ptr, other.__wbg_ptr);
        return U128.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U128}
     */
    divBigInt(other) {
        const ret = wasm.u128_divBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U128.__wrap(ret[0]);
    }
    /**
     * @param {U128} other
     * @returns {U128}
     */
    add(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_add(this.__wbg_ptr, other.__wbg_ptr);
        return U128.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U128}
     */
    addBigInt(other) {
        const ret = wasm.u128_addBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U128.__wrap(ret[0]);
    }
    /**
     * @param {U128} other
     * @returns {U128}
     */
    sub(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_sub(this.__wbg_ptr, other.__wbg_ptr);
        return U128.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U128}
     */
    subBigInt(other) {
        const ret = wasm.u128_subBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U128.__wrap(ret[0]);
    }
    /**
     * @param {U128} other
     * @returns {U128 | undefined}
     */
    checkedMul(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_checkedMul(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U128.__wrap(ret);
    }
    /**
     * @param {U128} other
     * @returns {U128 | undefined}
     */
    checkedAdd(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_checkedAdd(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U128.__wrap(ret);
    }
    /**
     * @param {U128} other
     * @returns {U128 | undefined}
     */
    checkedSub(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_checkedSub(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U128.__wrap(ret);
    }
    /**
     * @param {U128} other
     * @returns {U128 | undefined}
     */
    checkedDiv(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_checkedDiv(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U128.__wrap(ret);
    }
    /**
     * @param {U128} other
     * @returns {U128 | undefined}
     */
    checkedRem(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_checkedRem(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U128.__wrap(ret);
    }
    /**
     * @param {number} exp
     * @returns {U128 | undefined}
     */
    checkedPow(exp) {
        const ret = wasm.u128_checkedPow(this.__wbg_ptr, exp);
        return ret === 0 ? undefined : U128.__wrap(ret);
    }
    /**
     * @returns {bigint}
     */
    toBigInt() {
        const ret = wasm.u128_toBigInt(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {U128} other
     * @returns {boolean}
     */
    lt(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_lt(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U128} other
     * @returns {boolean}
     */
    le(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_le(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U128} other
     * @returns {boolean}
     */
    gt(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_gt(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U128} other
     * @returns {boolean}
     */
    ge(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_ge(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {string}
     */
    get value() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.u128_value(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {U128}
     */
    static MAX() {
        const ret = wasm.u128_MAX();
        return U128.__wrap(ret);
    }
    /**
     * @returns {U128}
     */
    static zero() {
        const ret = wasm.u128_zero();
        return U128.__wrap(ret);
    }
    /**
     * @param {U128} other
     * @returns {OverflowingResultU128}
     */
    overflowingMul(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_overflowingMul(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU128.__wrap(ret);
    }
    /**
     * @param {U128} other
     * @returns {OverflowingResultU128}
     */
    overflowingAdd(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_overflowingAdd(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU128.__wrap(ret);
    }
    /**
     * @param {U128} other
     * @returns {OverflowingResultU128}
     */
    overflowingSub(other) {
        _assertClass(other, U128);
        const ret = wasm.u128_overflowingSub(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU128.__wrap(ret);
    }
    /**
     * @param {number} exp
     * @returns {OverflowingResultU128}
     */
    overflowingPow(exp) {
        const ret = wasm.u128_overflowingPow(this.__wbg_ptr, exp);
        return OverflowingResultU128.__wrap(ret);
    }
    /**
     * @param {U512} value
     * @returns {U128}
     */
    static fromU512(value) {
        _assertClass(value, U512);
        var ptr0 = value.__destroy_into_raw();
        const ret = wasm.u128_fromU512(ptr0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U128.__wrap(ret[0]);
    }
    /**
     * @returns {U512}
     */
    toU512() {
        const ret = wasm.u128_toU512(this.__wbg_ptr);
        return U512.__wrap(ret);
    }
    /**
     * @param {U256} value
     * @returns {U128}
     */
    static fromU256(value) {
        _assertClass(value, U256);
        var ptr0 = value.__destroy_into_raw();
        const ret = wasm.u128_fromU256(ptr0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U128.__wrap(ret[0]);
    }
    /**
     * @returns {U256}
     */
    toU256() {
        const ret = wasm.u128_toU256(this.__wbg_ptr);
        return U256.__wrap(ret);
    }
}

const U256Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u256_free(ptr >>> 0, 1));

export class U256 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(U256.prototype);
        obj.__wbg_ptr = ptr;
        U256Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            value: this.value,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U256Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u256_free(ptr, 0);
    }
    /**
     * @param {string} value
     */
    constructor(value) {
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.u256_from_dec_str(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        U256Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} value
     * @returns {U256}
     */
    static fromNumber(value) {
        const ret = wasm.u256_fromNumber(value);
        return U256.__wrap(ret);
    }
    /**
     * @param {HTMLInputElement} input
     * @returns {U256}
     */
    static fromHtmlInput(input) {
        const ret = wasm.u256_fromHtmlInput(input);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U256.__wrap(ret[0]);
    }
    /**
     * @param {bigint} value
     * @returns {U256}
     */
    static fromBigInt(value) {
        const ret = wasm.u256_fromBigInt(value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U256.__wrap(ret[0]);
    }
    /**
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.u256_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.u256_toJson(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} decimals
     * @returns {BalanceFormatter}
     */
    formatter(decimals) {
        const ret = wasm.u256_formatter(this.__wbg_ptr, decimals);
        return BalanceFormatter.__wrap(ret);
    }
    /**
     * @param {U256} other
     * @returns {U256}
     */
    mul(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_mul(this.__wbg_ptr, other.__wbg_ptr);
        return U256.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U256}
     */
    mulBigInt(other) {
        const ret = wasm.u256_mulBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U256.__wrap(ret[0]);
    }
    /**
     * @param {U256} other
     * @returns {U256}
     */
    div(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_div(this.__wbg_ptr, other.__wbg_ptr);
        return U256.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U256}
     */
    divBigInt(other) {
        const ret = wasm.u256_divBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U256.__wrap(ret[0]);
    }
    /**
     * @param {U256} other
     * @returns {U256}
     */
    add(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_add(this.__wbg_ptr, other.__wbg_ptr);
        return U256.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U256}
     */
    addBigInt(other) {
        const ret = wasm.u256_addBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U256.__wrap(ret[0]);
    }
    /**
     * @param {U256} other
     * @returns {U256}
     */
    sub(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_sub(this.__wbg_ptr, other.__wbg_ptr);
        return U256.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U256}
     */
    subBigInt(other) {
        const ret = wasm.u256_subBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U256.__wrap(ret[0]);
    }
    /**
     * @param {U256} other
     * @returns {U256 | undefined}
     */
    checkedMul(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_checkedMul(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U256.__wrap(ret);
    }
    /**
     * @param {U256} other
     * @returns {U256 | undefined}
     */
    checkedAdd(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_checkedAdd(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U256.__wrap(ret);
    }
    /**
     * @param {U256} other
     * @returns {U256 | undefined}
     */
    checkedSub(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_checkedSub(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U256.__wrap(ret);
    }
    /**
     * @param {U256} other
     * @returns {U256 | undefined}
     */
    checkedDiv(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_checkedDiv(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U256.__wrap(ret);
    }
    /**
     * @param {U256} other
     * @returns {U256 | undefined}
     */
    checkedRem(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_checkedRem(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U256.__wrap(ret);
    }
    /**
     * @param {number} exp
     * @returns {U256 | undefined}
     */
    checkedPow(exp) {
        const ret = wasm.u256_checkedPow(this.__wbg_ptr, exp);
        return ret === 0 ? undefined : U256.__wrap(ret);
    }
    /**
     * @returns {bigint}
     */
    toBigInt() {
        const ret = wasm.u256_toBigInt(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {U256} other
     * @returns {boolean}
     */
    lt(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_lt(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U256} other
     * @returns {boolean}
     */
    le(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_le(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U256} other
     * @returns {boolean}
     */
    gt(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_gt(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U256} other
     * @returns {boolean}
     */
    ge(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_ge(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {string}
     */
    get value() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.u256_value(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {U256}
     */
    static MAX() {
        const ret = wasm.u256_MAX();
        return U256.__wrap(ret);
    }
    /**
     * @returns {U256}
     */
    static zero() {
        const ret = wasm.u256_zero();
        return U256.__wrap(ret);
    }
    /**
     * @param {U256} other
     * @returns {OverflowingResultU256}
     */
    overflowingMul(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_overflowingMul(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU256.__wrap(ret);
    }
    /**
     * @param {U256} other
     * @returns {OverflowingResultU256}
     */
    overflowingAdd(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_overflowingAdd(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU256.__wrap(ret);
    }
    /**
     * @param {U256} other
     * @returns {OverflowingResultU256}
     */
    overflowingSub(other) {
        _assertClass(other, U256);
        const ret = wasm.u256_overflowingSub(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU256.__wrap(ret);
    }
    /**
     * @param {number} exp
     * @returns {OverflowingResultU256}
     */
    overflowingPow(exp) {
        const ret = wasm.u256_overflowingPow(this.__wbg_ptr, exp);
        return OverflowingResultU256.__wrap(ret);
    }
    /**
     * @param {U512} value
     * @returns {U256}
     */
    static fromU512(value) {
        _assertClass(value, U512);
        var ptr0 = value.__destroy_into_raw();
        const ret = wasm.u256_fromU512(ptr0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U256.__wrap(ret[0]);
    }
    /**
     * @returns {U512}
     */
    toU512() {
        const ret = wasm.u256_toU512(this.__wbg_ptr);
        return U512.__wrap(ret);
    }
}

const U512Finalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_u512_free(ptr >>> 0, 1));

export class U512 {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(U512.prototype);
        obj.__wbg_ptr = ptr;
        U512Finalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    toJSON() {
        return {
            value: this.value,
        };
    }

    toString() {
        return JSON.stringify(this);
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        U512Finalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_u512_free(ptr, 0);
    }
    /**
     * @param {string} value
     */
    constructor(value) {
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.u512_from_dec_str(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        U512Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} value
     * @returns {U512}
     */
    static fromNumber(value) {
        const ret = wasm.u512_fromNumber(value);
        return U512.__wrap(ret);
    }
    /**
     * @param {HTMLInputElement} input
     * @returns {U512}
     */
    static fromHtmlInput(input) {
        const ret = wasm.u512_fromHtmlInput(input);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U512.__wrap(ret[0]);
    }
    /**
     * @param {bigint} value
     * @returns {U512}
     */
    static fromBigInt(value) {
        const ret = wasm.u512_fromBigInt(value);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U512.__wrap(ret[0]);
    }
    /**
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.u512_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.u512_toJson(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} decimals
     * @returns {BalanceFormatter}
     */
    formatter(decimals) {
        const ret = wasm.u512_formatter(this.__wbg_ptr, decimals);
        return BalanceFormatter.__wrap(ret);
    }
    /**
     * @param {U512} other
     * @returns {U512}
     */
    mul(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_mul(this.__wbg_ptr, other.__wbg_ptr);
        return U512.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U512}
     */
    mulBigInt(other) {
        const ret = wasm.u512_mulBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U512.__wrap(ret[0]);
    }
    /**
     * @param {U512} other
     * @returns {U512}
     */
    div(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_div(this.__wbg_ptr, other.__wbg_ptr);
        return U512.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U512}
     */
    divBigInt(other) {
        const ret = wasm.u512_divBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U512.__wrap(ret[0]);
    }
    /**
     * @param {U512} other
     * @returns {U512}
     */
    add(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_add(this.__wbg_ptr, other.__wbg_ptr);
        return U512.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U512}
     */
    addBigInt(other) {
        const ret = wasm.u512_addBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U512.__wrap(ret[0]);
    }
    /**
     * @param {U512} other
     * @returns {U512}
     */
    sub(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_sub(this.__wbg_ptr, other.__wbg_ptr);
        return U512.__wrap(ret);
    }
    /**
     * @param {bigint} other
     * @returns {U512}
     */
    subBigInt(other) {
        const ret = wasm.u512_subBigInt(this.__wbg_ptr, other);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return U512.__wrap(ret[0]);
    }
    /**
     * @param {U512} other
     * @returns {U512 | undefined}
     */
    checkedMul(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_checkedMul(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U512.__wrap(ret);
    }
    /**
     * @param {U512} other
     * @returns {U512 | undefined}
     */
    checkedAdd(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_checkedAdd(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U512.__wrap(ret);
    }
    /**
     * @param {U512} other
     * @returns {U512 | undefined}
     */
    checkedSub(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_checkedSub(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U512.__wrap(ret);
    }
    /**
     * @param {U512} other
     * @returns {U512 | undefined}
     */
    checkedDiv(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_checkedDiv(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U512.__wrap(ret);
    }
    /**
     * @param {U512} other
     * @returns {U512 | undefined}
     */
    checkedRem(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_checkedRem(this.__wbg_ptr, other.__wbg_ptr);
        return ret === 0 ? undefined : U512.__wrap(ret);
    }
    /**
     * @param {number} exp
     * @returns {U512 | undefined}
     */
    checkedPow(exp) {
        const ret = wasm.u512_checkedPow(this.__wbg_ptr, exp);
        return ret === 0 ? undefined : U512.__wrap(ret);
    }
    /**
     * @returns {bigint}
     */
    toBigInt() {
        const ret = wasm.u512_toBigInt(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {U512} other
     * @returns {boolean}
     */
    lt(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_lt(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U512} other
     * @returns {boolean}
     */
    le(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_le(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U512} other
     * @returns {boolean}
     */
    gt(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_gt(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {U512} other
     * @returns {boolean}
     */
    ge(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_ge(this.__wbg_ptr, other.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {string}
     */
    get value() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.u512_value(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {U512}
     */
    static MAX() {
        const ret = wasm.u512_MAX();
        return U512.__wrap(ret);
    }
    /**
     * @returns {U512}
     */
    static zero() {
        const ret = wasm.u512_zero();
        return U512.__wrap(ret);
    }
    /**
     * @param {U512} other
     * @returns {OverflowingResultU512}
     */
    overflowingMul(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_overflowingMul(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU512.__wrap(ret);
    }
    /**
     * @param {U512} other
     * @returns {OverflowingResultU512}
     */
    overflowingAdd(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_overflowingAdd(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU512.__wrap(ret);
    }
    /**
     * @param {U512} other
     * @returns {OverflowingResultU512}
     */
    overflowingSub(other) {
        _assertClass(other, U512);
        const ret = wasm.u512_overflowingSub(this.__wbg_ptr, other.__wbg_ptr);
        return OverflowingResultU512.__wrap(ret);
    }
    /**
     * @param {number} exp
     * @returns {OverflowingResultU512}
     */
    overflowingPow(exp) {
        const ret = wasm.u512_overflowingPow(this.__wbg_ptr, exp);
        return OverflowingResultU512.__wrap(ret);
    }
}

const URefFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_uref_free(ptr >>> 0, 1));

export class URef {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(URef.prototype);
        obj.__wbg_ptr = ptr;
        URefFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        URefFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_uref_free(ptr, 0);
    }
    /**
     * @param {string} uref_hex_str
     * @param {number} access_rights
     */
    constructor(uref_hex_str, access_rights) {
        const ptr0 = passStringToWasm0(uref_hex_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uref_new_js_alias(ptr0, len0, access_rights);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        URefFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {string} formatted_str
     * @returns {URef}
     */
    static fromFormattedStr(formatted_str) {
        const ptr0 = passStringToWasm0(formatted_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uref_fromFormattedStr(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return URef.__wrap(ret[0]);
    }
    /**
     * @param {Uint8Array} bytes
     * @param {number} access_rights
     * @returns {URef}
     */
    static fromUint8Array(bytes, access_rights) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.uref_fromUint8Array(ptr0, len0, access_rights);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return URef.__wrap(ret[0]);
    }
    /**
     * @returns {string}
     */
    toFormattedString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.uref_toFormattedString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.uref_toJson(this.__wbg_ptr);
        return ret;
    }
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_BigInt_ddea6d2f55558acb = function() { return handleError(function (arg0) {
        const ret = BigInt(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_abort_410ec47a64ac6117 = function(arg0, arg1) {
        arg0.abort(arg1);
    };
    imports.wbg.__wbg_abort_775ef1d17fc65868 = function(arg0) {
        arg0.abort();
    };
    imports.wbg.__wbg_accountinfo_new = function(arg0) {
        const ret = AccountInfo.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_addEventListener_90e553fdce254421 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        arg0.addEventListener(getStringFromWasm0(arg1, arg2), arg3);
    }, arguments) };
    imports.wbg.__wbg_address_new = function(arg0) {
        const ret = Address.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_append_8c7dd8d641a5f01b = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        arg0.append(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
    }, arguments) };
    imports.wbg.__wbg_arrayBuffer_d1b44c4390db422f = function() { return handleError(function (arg0) {
        const ret = arg0.arrayBuffer();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_buffer_609cc3eee51ed158 = function(arg0) {
        const ret = arg0.buffer;
        return ret;
    };
    imports.wbg.__wbg_call_672a4d21634d4a24 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_7cccdd69e0791ae2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_833bed5770ea2041 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.call(arg1, arg2, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_clearTimeout_42d9ccd50822fd3a = function(arg0) {
        const ret = clearTimeout(arg0);
        return ret;
    };
    imports.wbg.__wbg_connect_1b07649fef0234a9 = function() { return handleError(function (arg0, arg1) {
        const ret = window.csprclick.connect(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_contracts_new = function(arg0) {
        const ret = Contracts.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_disconnect_d38ecd0a3f8d5b41 = function() { return handleError(function () {
        const ret = window.csprclick.disconnect();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_done_769e5ede4b31c67b = function(arg0) {
        const ret = arg0.done;
        return ret;
    };
    imports.wbg.__wbg_fetch_1b7e793ab8320753 = function(arg0, arg1, arg2) {
        const ret = arg0.fetch(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_fetch_509096533071c657 = function(arg0, arg1) {
        const ret = arg0.fetch(arg1);
        return ret;
    };
    imports.wbg.__wbg_fetch_6bbc32f991730587 = function(arg0) {
        const ret = fetch(arg0);
        return ret;
    };
    imports.wbg.__wbg_getActiveAccountAsync_2158257d8ec4db96 = function() { return handleError(function (arg0) {
        const ret = window.csprclick.getActiveAccountAsync(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getActivePublicKey_21fe271c15317cc9 = function() { return handleError(function () {
        const ret = window.csprclick.getActivePublicKey();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_get_67b2ba62fc30de12 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.get(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getindex_5b00c274b05714aa = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return ret;
    };
    imports.wbg.__wbg_has_a5ea9117f258a0ec = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.has(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_headers_9cb51cfd2ac780a4 = function(arg0) {
        const ret = arg0.headers;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Response_f2cc20d9f7dfd644 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Response;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Window_def73ea0955fc569 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Window;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_isUnlocked_bdce9e15fa797fc3 = function() { return handleError(function (arg0, arg1) {
        const ret = window.csprclick.isUnlocked(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_iterator_9a24c88df860dc65 = function() {
        const ret = Symbol.iterator;
        return ret;
    };
    imports.wbg.__wbg_json_1671bfa3e3625686 = function() { return handleError(function (arg0) {
        const ret = arg0.json();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_length_a446193dc22c12f8 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_log_453075a99785c891 = function(arg0, arg1) {
        console.log(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbg_makesupplierconfig_new = function(arg0) {
        const ret = MakeSupplierConfig.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_new0_f788a2397c7ca929 = function() {
        const ret = new Date();
        return ret;
    };
    imports.wbg.__wbg_new_018dcc2d6c8c2f6a = function() { return handleError(function () {
        const ret = new Headers();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_new_23a2665fac83c611 = function(arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = (arg0, arg1) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_480(a, state0.b, arg0, arg1);
                } finally {
                    state0.a = a;
                }
            };
            const ret = new Promise(cb0);
            return ret;
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_new_405e22f390576ce2 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_a12002a7f91c75be = function(arg0) {
        const ret = new Uint8Array(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_e25e5aab09ff45db = function() { return handleError(function () {
        const ret = new AbortController();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_newnoargs_105ed471475aaf50 = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_d97e637ebe145a9a = function(arg0, arg1, arg2) {
        const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_newwithstrandinit_06c535e0a867c635 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = new Request(getStringFromWasm0(arg0, arg1), arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_next_25feadfc0913fea9 = function(arg0) {
        const ret = arg0.next;
        return ret;
    };
    imports.wbg.__wbg_next_6574e1a8a62d1055 = function() { return handleError(function (arg0) {
        const ret = arg0.next();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_on_603e52d4683f4932 = function() { return handleError(function (arg0, arg1, arg2) {
        window.csprclick.on(getStringFromWasm0(arg0, arg1), arg2);
    }, arguments) };
    imports.wbg.__wbg_parse_def2e24ef1252aff = function() { return handleError(function (arg0, arg1) {
        const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_queueMicrotask_97d92b4fcc8a61c5 = function(arg0) {
        queueMicrotask(arg0);
    };
    imports.wbg.__wbg_queueMicrotask_d3219def82552485 = function(arg0) {
        const ret = arg0.queueMicrotask;
        return ret;
    };
    imports.wbg.__wbg_resolve_4851785c9c5f573d = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_send_0ec9803e773e3faf = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = window.csprclick.send(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3), arg4);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_setTimeout_4ec014681668a581 = function(arg0, arg1) {
        const ret = setTimeout(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_set_65595bdd868b3009 = function(arg0, arg1, arg2) {
        arg0.set(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_setbody_5923b78a95eedf29 = function(arg0, arg1) {
        arg0.body = arg1;
    };
    imports.wbg.__wbg_setcache_12f17c3a980650e4 = function(arg0, arg1) {
        arg0.cache = __wbindgen_enum_RequestCache[arg1];
    };
    imports.wbg.__wbg_setcredentials_c3a22f1cd105a2c6 = function(arg0, arg1) {
        arg0.credentials = __wbindgen_enum_RequestCredentials[arg1];
    };
    imports.wbg.__wbg_setheaders_834c0bdb6a8949ad = function(arg0, arg1) {
        arg0.headers = arg1;
    };
    imports.wbg.__wbg_setmethod_3c5280fe5d890842 = function(arg0, arg1, arg2) {
        arg0.method = getStringFromWasm0(arg1, arg2);
    };
    imports.wbg.__wbg_setmode_5dc300b865044b65 = function(arg0, arg1) {
        arg0.mode = __wbindgen_enum_RequestMode[arg1];
    };
    imports.wbg.__wbg_setsignal_75b21ef3a81de905 = function(arg0, arg1) {
        arg0.signal = arg1;
    };
    imports.wbg.__wbg_signInWithAccount_d533cbb722c84e2e = function() { return handleError(function (arg0) {
        const ret = window.csprclick.signInWithAccount(AccountInfo.__wrap(arg0));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_signIn_b02d46290a9f4a91 = function() { return handleError(function () {
        window.csprclick.signIn();
    }, arguments) };
    imports.wbg.__wbg_signMessage_df25336b785f84d0 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = window.csprclick.signMessage(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_signOut_bcc043272ae9fc4f = function() { return handleError(function () {
        window.csprclick.signOut();
    }, arguments) };
    imports.wbg.__wbg_signal_aaf9ad74119f20a4 = function(arg0) {
        const ret = arg0.signal;
        return ret;
    };
    imports.wbg.__wbg_signresult_new = function(arg0) {
        const ret = SignResult.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_88a902d13a557d07 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0 = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_37c5d418e4bf5819 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_5de37043a91a9c40 = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_status_f6360336ca686bf0 = function(arg0) {
        const ret = arg0.status;
        return ret;
    };
    imports.wbg.__wbg_stringify_f7ed6987935b4a24 = function() { return handleError(function (arg0) {
        const ret = JSON.stringify(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_styksblockysupplerconfig_new = function(arg0) {
        const ret = StyksBlockySupplerConfig.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_stykspricefeedconfig_new = function(arg0) {
        const ret = StyksPriceFeedConfig.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_switchAccount_abc1cd2c36fdd8e6 = function() { return handleError(function () {
        const ret = window.csprclick.switchAccount();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_then_44b73946d2fb3e7d = function(arg0, arg1) {
        const ret = arg0.then(arg1);
        return ret;
    };
    imports.wbg.__wbg_then_48b406749878a531 = function(arg0, arg1, arg2) {
        const ret = arg0.then(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_toISOString_b015155a5a6fe219 = function(arg0) {
        const ret = arg0.toISOString();
        return ret;
    };
    imports.wbg.__wbg_toString_b5d4438bc26b267c = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.toString(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_transactionresult_new = function(arg0) {
        const ret = TransactionResult.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_u512_new = function(arg0) {
        const ret = U512.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_url_ae10c34ca209681d = function(arg0, arg1) {
        const ret = arg1.url;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_91cbf0dd3ab84c1e = function(arg0, arg1) {
        const ret = arg1.value;
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbg_value_cd1ffa7b1ab794f1 = function(arg0) {
        const ret = arg0.value;
        return ret;
    };
    imports.wbg.__wbindgen_array_new = function() {
        const ret = [];
        return ret;
    };
    imports.wbg.__wbindgen_array_push = function(arg0, arg1) {
        arg0.push(arg1);
    };
    imports.wbg.__wbindgen_bigint_from_u64 = function(arg0) {
        const ret = BigInt.asUintN(64, arg0);
        return ret;
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = arg0;
        const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        return ret;
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = arg0.original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        const ret = false;
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper1766 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 362, __wbg_adapter_42);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper1767 = function(arg0, arg1, arg2) {
        const ret = makeClosure(arg0, arg1, 362, __wbg_adapter_45);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper3342 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 792, __wbg_adapter_48);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper3370 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 809, __wbg_adapter_51);
        return ret;
    };
    imports.wbg.__wbindgen_debug_string = function(arg0, arg1) {
        const ret = debugString(arg1);
        const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_error_new = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_2;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(arg0) === 'function';
        return ret;
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = arg0;
        const ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return ret;
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_rethrow = function(arg0) {
        throw arg0;
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_uint8_array_new = function(arg0, arg1) {
        var v0 = getArrayU8FromWasm0(arg0, arg1).slice();
        wasm.__wbindgen_free(arg0, arg1 * 1, 1);
        const ret = v0;
        return ret;
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('styks_wasm_client_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
