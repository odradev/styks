let wasm;

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

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_2.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
function __wbg_adapter_38(arg0, arg1) {
    wasm._dyn_core__ops__function__FnMut_____Output___R_as_wasm_bindgen__closure__WasmClosure___describe__invoke__heabe0ed6d136b696(arg0, arg1);
}

function __wbg_adapter_41(arg0, arg1, arg2) {
    wasm.closure877_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_286(arg0, arg1, arg2, arg3) {
    wasm.closure1394_externref_shim(arg0, arg1, arg2, arg3);
}

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

const AccessRightsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_accessrights_free(ptr >>> 0, 1));

export class AccessRights {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(AccessRights.prototype);
        obj.__wbg_ptr = ptr;
        AccessRightsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        AccessRightsFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_accessrights_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    static NONE() {
        const ret = wasm.accessrights_NONE();
        return ret;
    }
    /**
     * @returns {number}
     */
    static READ() {
        const ret = wasm.accessrights_READ();
        return ret;
    }
    /**
     * @returns {number}
     */
    static WRITE() {
        const ret = wasm.accessrights_WRITE();
        return ret;
    }
    /**
     * @returns {number}
     */
    static ADD() {
        const ret = wasm.accessrights_ADD();
        return ret;
    }
    /**
     * @returns {number}
     */
    static READ_ADD() {
        const ret = wasm.accessrights_READ_ADD();
        return ret;
    }
    /**
     * @returns {number}
     */
    static READ_WRITE() {
        const ret = wasm.accessrights_READ_WRITE();
        return ret;
    }
    /**
     * @returns {number}
     */
    static ADD_WRITE() {
        const ret = wasm.accessrights_ADD_WRITE();
        return ret;
    }
    /**
     * @returns {number}
     */
    static READ_ADD_WRITE() {
        const ret = wasm.accessrights_READ_ADD_WRITE();
        return ret;
    }
    /**
     * @param {number} access_rights
     */
    constructor(access_rights) {
        const ret = wasm.accessrights_new(access_rights);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        AccessRightsFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {boolean} read
     * @param {boolean} write
     * @param {boolean} add
     * @returns {AccessRights}
     */
    static from_bits(read, write, add) {
        const ret = wasm.accessrights_from_bits(read, write, add);
        return AccessRights.__wrap(ret);
    }
    /**
     * @returns {boolean}
     */
    is_readable() {
        const ret = wasm.accessrights_is_readable(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    is_writeable() {
        const ret = wasm.accessrights_is_writeable(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    is_addable() {
        const ret = wasm.accessrights_is_addable(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    is_none() {
        const ret = wasm.accessrights_is_none(this.__wbg_ptr);
        return ret !== 0;
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
}

const CasperWalletFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_casperwallet_free(ptr >>> 0, 1));

export class CasperWallet {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        CasperWalletFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_casperwallet_free(ptr, 0);
    }
    constructor() {
        const ret = wasm.casperwallet_new();
        this.__wbg_ptr = ret >>> 0;
        CasperWalletFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Deploy} deploy
     * @param {string | null} [public_key]
     * @returns {Promise<Deploy>}
     */
    signDeploy(deploy, public_key) {
        _assertClass(deploy, Deploy);
        var ptr0 = deploy.__destroy_into_raw();
        var ptr1 = isLikeNone(public_key) ? 0 : passStringToWasm0(public_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        const ret = wasm.casperwallet_signDeploy(this.__wbg_ptr, ptr0, ptr1, len1);
        return ret;
    }
    /**
     * @param {Transaction} transaction
     * @param {string | null} [public_key]
     * @returns {Promise<Transaction>}
     */
    signTransaction(transaction, public_key) {
        _assertClass(transaction, Transaction);
        var ptr0 = transaction.__destroy_into_raw();
        var ptr1 = isLikeNone(public_key) ? 0 : passStringToWasm0(public_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        const ret = wasm.casperwallet_signTransaction(this.__wbg_ptr, ptr0, ptr1, len1);
        return ret;
    }
    /**
     * Alias for the `sign_message` function, specifically for signing transaction hashes.
     *
     * This function calls `sign_message` to sign the provided transaction hash with the
     * given or active public key.
     *
     * # Arguments
     *
     * * `transaction_hash` - The transaction hash string to be signed.
     * * `public_key` - An optional public key string. If `None`, the active public key is used.
     *
     * # Returns
     *
     * * `Ok(String)` - The signature string.
     * * `Err(JsError)` - An error if the signing process fails.
     * @param {string} transaction_hash
     * @param {string | null} [public_key]
     * @returns {Promise<string>}
     */
    signTransactionHash(transaction_hash, public_key) {
        const ptr0 = passStringToWasm0(transaction_hash, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(public_key) ? 0 : passStringToWasm0(public_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        const ret = wasm.casperwallet_signTransactionHash(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * Signs a message with the provided or active public key.
     *
     * This function requests a connection to the wallet, retrieves the public key
     * (either provided or active), signs the message, and returns the signature.
     *
     * # Arguments
     *
     * * `message` - The message string to be signed.
     * * `public_key` - An optional public key string. If `None`, the active public key is used.
     *
     * # Returns
     *
     * * `Ok(String)` - The signature string.
     * * `Err(JsError)` - An error if the connection fails, the public key retrieval fails,
     *   the signing fails, or if the signing is cancelled.
     *
     * # Errors
     *
     * This function returns a `JsError` if:
     * * The connection to the wallet could not be established.
     * * The public key could not be retrieved.
     * * The signing operation fails.
     * * The signing is cancelled by the user.
     * @param {string} message
     * @param {string | null} [public_key]
     * @returns {Promise<string>}
     */
    signMessage(message, public_key) {
        const ptr0 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        var ptr1 = isLikeNone(public_key) ? 0 : passStringToWasm0(public_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        const ret = wasm.casperwallet_signMessage(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return ret;
    }
    /**
     * @returns {Promise<void>}
     */
    connect() {
        const ret = wasm.casperwallet_connect(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Promise<boolean>}
     */
    disconnect() {
        const ret = wasm.casperwallet_disconnect(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Promise<boolean>}
     */
    isConnected() {
        const ret = wasm.casperwallet_isConnected(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Promise<string>}
     */
    getVersion() {
        const ret = wasm.casperwallet_getVersion(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Promise<string>}
     */
    getActivePublicKey() {
        const ret = wasm.casperwallet_getActivePublicKey(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Promise<boolean>}
     */
    switchAccount() {
        const ret = wasm.casperwallet_switchAccount(this.__wbg_ptr);
        return ret;
    }
}

const DeployFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_deploy_free(ptr >>> 0, 1));

export class Deploy {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Deploy.prototype);
        obj.__wbg_ptr = ptr;
        DeployFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DeployFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_deploy_free(ptr, 0);
    }
    /**
     * @param {string} public_key
     * @param {string} signature
     * @returns {Deploy}
     */
    add_signature(public_key, signature) {
        const ptr0 = passStringToWasm0(public_key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(signature, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.deploy_add_signature(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return Deploy.__wrap(ret);
    }
}

const DigestFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_digest_free(ptr >>> 0, 1));

export class Digest {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Digest.prototype);
        obj.__wbg_ptr = ptr;
        DigestFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DigestFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_digest_free(ptr, 0);
    }
    /**
     * @param {string} digest_hex_str
     */
    constructor(digest_hex_str) {
        const ptr0 = passStringToWasm0(digest_hex_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.digest_new_js_alias(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        DigestFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {string} digest_hex_str
     * @returns {Digest}
     */
    static fromString(digest_hex_str) {
        const ptr0 = passStringToWasm0(digest_hex_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.digest_fromString(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Digest.__wrap(ret[0]);
    }
    /**
     * @param {Uint8Array} bytes
     * @returns {Digest}
     */
    static fromRaw(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.digest_fromRaw(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Digest.__wrap(ret[0]);
    }
    /**
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.digest_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}

const OdraWasmClientFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_odrawasmclient_free(ptr >>> 0, 1));

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
     * @param {bigint | null} [gas]
     * @param {number | null} [ttl]
     * @param {Verbosity | null} [verbosity]
     */
    constructor(node_address, speculative_node_address, chain_name, gas, ttl, verbosity) {
        const ptr0 = passStringToWasm0(node_address, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(speculative_node_address, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(chain_name) ? 0 : passStringToWasm0(chain_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len2 = WASM_VECTOR_LEN;
        const ret = wasm.odrawasmclient_new(ptr0, len0, ptr1, len1, ptr2, len2, !isLikeNone(gas), isLikeNone(gas) ? BigInt(0) : gas, isLikeNone(ttl) ? 0x100000001 : (ttl) >>> 0, isLikeNone(verbosity) ? 3 : verbosity);
        this.__wbg_ptr = ret >>> 0;
        OdraWasmClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {bigint} gas
     */
    setGas(gas) {
        wasm.odrawasmclient_setGas(this.__wbg_ptr, gas);
    }
    /**
     * @returns {bigint}
     */
    static DEFAULT_PAYMENT() {
        const ret = wasm.odrawasmclient_DEFAULT_PAYMENT();
        return BigInt.asUintN(64, ret);
    }
    /**
     * Gets a value from a named key of an account or a contract
     * @param {Address} address
     * @param {string} name
     * @returns {Promise<Bytes | undefined>}
     */
    getNamedValue(address, name) {
        _assertClass(address, Address);
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.odrawasmclient_getNamedValue(this.__wbg_ptr, address.__wbg_ptr, ptr0, len0);
        return ret;
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
        return ret;
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
        this.__wbg_ptr = ret >>> 0;
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
        this.__wbg_ptr = ret >>> 0;
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
        const ret = wasm.rolegranted_new(ptr0, len0, ptr1, ptr2);
        this.__wbg_ptr = ret >>> 0;
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
        const ret = wasm.__wbg_get_styksblockysupplerconfig_timestampTolerance(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set timestampTolerance(arg0) {
        wasm.__wbg_set_styksblockysupplerconfig_timestampTolerance(this.__wbg_ptr, arg0);
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
        this.__wbg_ptr = ret >>> 0;
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
        var ptr0 = wasmClient.__destroy_into_raw();
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_new(ptr0, ptr1);
        this.__wbg_ptr = ret >>> 0;
        StyksBlockySupplierWasmClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {StyksBlockySupplerConfig} config
     * @returns {Promise<TransactionHash>}
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
     * @param {Uint8Array} signature
     * @param {Uint8Array} data
     * @returns {Promise<TransactionHash>}
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
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionHash>}
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
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionHash>}
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
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionHash>}
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
        const ret = wasm.__wbg_get_styksblockysupplerconfig_timestampTolerance(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set heartbeatInterval(arg0) {
        wasm.__wbg_set_styksblockysupplerconfig_timestampTolerance(this.__wbg_ptr, arg0);
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
        this.__wbg_ptr = ret >>> 0;
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
        var ptr0 = wasmClient.__destroy_into_raw();
        _assertClass(address, Address);
        var ptr1 = address.__destroy_into_raw();
        const ret = wasm.styksblockysupplierwasmclient_new(ptr0, ptr1);
        this.__wbg_ptr = ret >>> 0;
        StyksPriceFeedWasmClientFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {StyksPriceFeedConfig} config
     * @returns {Promise<TransactionHash>}
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
     * @returns {Promise<TransactionHash>}
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
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionHash>}
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
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionHash>}
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
     * @param {Uint8Array} role
     * @param {Address} address
     * @returns {Promise<TransactionHash>}
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

const TransactionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_transaction_free(ptr >>> 0, 1));

export class Transaction {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(Transaction.prototype);
        obj.__wbg_ptr = ptr;
        TransactionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TransactionFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_transaction_free(ptr, 0);
    }
    /**
     * @param {any} transaction
     */
    constructor(transaction) {
        const ret = wasm.transaction_new(transaction);
        this.__wbg_ptr = ret >>> 0;
        TransactionFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.transaction_toJson(this.__wbg_ptr);
        return ret;
    }
}

const TransactionHashFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_transactionhash_free(ptr >>> 0, 1));

export class TransactionHash {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TransactionHash.prototype);
        obj.__wbg_ptr = ptr;
        TransactionHashFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TransactionHashFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_transactionhash_free(ptr, 0);
    }
    /**
     * @param {string} transaction_hash_hex_str
     */
    constructor(transaction_hash_hex_str) {
        const ptr0 = passStringToWasm0(transaction_hash_hex_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.transactionhash_new_js_alias(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        TransactionHashFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {Uint8Array} bytes
     * @returns {TransactionHash}
     */
    static fromRaw(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.transactionhash_fromRaw(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return TransactionHash.__wrap(ret[0]);
    }
    /**
     * @returns {Digest}
     */
    digest() {
        const ret = wasm.transactionhash_digest(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return Digest.__wrap(ret[0]);
    }
    /**
     * @returns {any}
     */
    toJson() {
        const ret = wasm.transactionhash_toJson(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    toString() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.transactionhash_toString(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
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
        this.__wbg_ptr = ret >>> 0;
        U128Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} value
     * @returns {U128}
     */
    static fromU32(value) {
        const ret = wasm.u128_fromU32(value);
        return U128.__wrap(ret);
    }
    /**
     * @param {bigint} value
     * @returns {U128}
     */
    static fromBigInt(value) {
        const ret = wasm.u128_fromBigInt(value);
        return U128.__wrap(ret);
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
        this.__wbg_ptr = ret >>> 0;
        U256Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} value
     * @returns {U256}
     */
    static fromU32(value) {
        const ret = wasm.u256_fromU32(value);
        return U256.__wrap(ret);
    }
    /**
     * @param {bigint} value
     * @returns {U256}
     */
    static fromBigInt(value) {
        const ret = wasm.u256_fromBigInt(value);
        return U256.__wrap(ret);
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
        this.__wbg_ptr = ret >>> 0;
        U512Finalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} value
     * @returns {U512}
     */
    static fromU32(value) {
        const ret = wasm.u512_fromU32(value);
        return U512.__wrap(ret);
    }
    /**
     * @param {bigint} value
     * @returns {U512}
     */
    static fromBigInt(value) {
        const ret = wasm.u512_fromBigInt(value);
        return U512.__wrap(ret);
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
        return URef.__wrap(ret);
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

const URefAddrFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_urefaddr_free(ptr >>> 0, 1));

export class URefAddr {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        URefAddrFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_urefaddr_free(ptr, 0);
    }
    /**
     * @param {Uint8Array} bytes
     */
    constructor(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.urefaddr_new(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        URefAddrFinalization.register(this, this.__wbg_ptr, this);
        return this;
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
    imports.wbg.__wbg_CasperWalletProvider_ab523e0e01b76171 = function() {
        const ret = CasperWalletProvider();
        return ret;
    };
    imports.wbg.__wbg_abort_410ec47a64ac6117 = function(arg0, arg1) {
        arg0.abort(arg1);
    };
    imports.wbg.__wbg_abort_775ef1d17fc65868 = function(arg0) {
        arg0.abort();
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
    imports.wbg.__wbg_bytes_new = function(arg0) {
        const ret = Bytes.__wrap(arg0);
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
    imports.wbg.__wbg_clearTimeout_6222fede17abcb1a = function(arg0) {
        const ret = clearTimeout(arg0);
        return ret;
    };
    imports.wbg.__wbg_deploy_new = function(arg0) {
        const ret = Deploy.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_disconnectFromSite_6bf09186bad7427f = function() { return handleError(function (arg0) {
        const ret = arg0.disconnectFromSite();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_done_769e5ede4b31c67b = function(arg0) {
        const ret = arg0.done;
        return ret;
    };
    imports.wbg.__wbg_fetch_509096533071c657 = function(arg0, arg1) {
        const ret = arg0.fetch(arg1);
        return ret;
    };
    imports.wbg.__wbg_fetch_f156d10be9a5c88a = function(arg0) {
        const ret = fetch(arg0);
        return ret;
    };
    imports.wbg.__wbg_getActivePublicKey_38051465ad5b9163 = function() { return handleError(function (arg0) {
        const ret = arg0.getActivePublicKey();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_getVersion_f53d75f59825127f = function() { return handleError(function (arg0) {
        const ret = arg0.getVersion();
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
    imports.wbg.__wbg_isConnected_79e9895d50624c0e = function() { return handleError(function (arg0) {
        const ret = arg0.isConnected();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_iterator_9a24c88df860dc65 = function() {
        const ret = Symbol.iterator;
        return ret;
    };
    imports.wbg.__wbg_length_a446193dc22c12f8 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_log_33938a6bbdacf5f3 = function(arg0, arg1) {
        console.log(getStringFromWasm0(arg0, arg1));
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
                    return __wbg_adapter_286(a, state0.b, arg0, arg1);
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
    imports.wbg.__wbg_requestConnection_64ae8e88402fa051 = function() { return handleError(function (arg0) {
        const ret = arg0.requestConnection();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_requestSwitchAccount_2b9dabdb19659441 = function() { return handleError(function (arg0) {
        const ret = arg0.requestSwitchAccount();
        return ret;
    }, arguments) };
    imports.wbg.__wbg_resolve_4851785c9c5f573d = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_setTimeout_2b339866a2aa3789 = function(arg0, arg1) {
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
    imports.wbg.__wbg_signMessage_f76f79f81d15a623 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.signMessage(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_sign_bc717d862187827b = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.sign(getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_signal_aaf9ad74119f20a4 = function(arg0) {
        const ret = arg0.signal;
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
    imports.wbg.__wbg_transaction_new = function(arg0) {
        const ret = Transaction.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_transactionhash_new = function(arg0) {
        const ret = TransactionHash.__wrap(arg0);
        return ret;
    };
    imports.wbg.__wbg_url_ae10c34ca209681d = function(arg0, arg1) {
        const ret = arg1.url;
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
    imports.wbg.__wbindgen_closure_wrapper3072 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 842, __wbg_adapter_38);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper3133 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 878, __wbg_adapter_41);
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
