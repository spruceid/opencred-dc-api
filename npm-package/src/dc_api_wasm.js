
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextDecoder, TextEncoder } = require(`util`);

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = new TextEncoder('utf-8');

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

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_4.set(idx, obj);
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

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_6.get(state.dtor)(state.a, state.b)
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
                wasm.__wbindgen_export_6.get(state.dtor)(a, state.b);
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

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_4.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
function __wbg_adapter_52(arg0, arg1, arg2) {
    wasm.closure1630_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_149(arg0, arg1, arg2, arg3) {
    wasm.closure1993_externref_shim(arg0, arg1, arg2, arg3);
}

const DcApiFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_dcapi_free(ptr >>> 0, 1));

class DcApi {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(DcApi.prototype);
        obj.__wbg_ptr = ptr;
        DcApiFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        DcApiFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_dcapi_free(ptr, 0);
    }
    /**
     * @param {string} key
     * @param {string} base_url
     * @param {string} submission_endpoint
     * @param {string} reference_endpoint
     * @param {Uint8Array} cert_chain_pem
     * @param {JsOid4VpSessionStore} oid4vp_session_store
     * @param {DcApiSessionStore} js_dc_api_session_store
     * @returns {Promise<DcApi>}
     */
    static new(key, base_url, submission_endpoint, reference_endpoint, cert_chain_pem, oid4vp_session_store, js_dc_api_session_store) {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(base_url, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(submission_endpoint, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(reference_endpoint, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ptr4 = passArray8ToWasm0(cert_chain_pem, wasm.__wbindgen_malloc);
        const len4 = WASM_VECTOR_LEN;
        _assertClass(oid4vp_session_store, JsOid4VpSessionStore);
        var ptr5 = oid4vp_session_store.__destroy_into_raw();
        const ret = wasm.dcapi_new(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4, ptr5, js_dc_api_session_store);
        return ret;
    }
    /**
     * @returns {Promise<any>}
     */
    create_new_session() {
        const ret = wasm.dcapi_create_new_session(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {string} session_id
     * @param {string} session_secret
     * @param {any} request
     * @param {string | null} [user_agent]
     * @returns {Promise<any>}
     */
    initiate_request(session_id, session_secret, request, user_agent) {
        const ptr0 = passStringToWasm0(session_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(session_secret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        var ptr2 = isLikeNone(user_agent) ? 0 : passStringToWasm0(user_agent, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len2 = WASM_VECTOR_LEN;
        const ret = wasm.dcapi_initiate_request(this.__wbg_ptr, ptr0, len0, ptr1, len1, request, ptr2, len2);
        return ret;
    }
    /**
     * @param {string} session_id
     * @param {string} session_secret
     * @param {any} response
     * @returns {Promise<any>}
     */
    submit_response(session_id, session_secret, response) {
        const ptr0 = passStringToWasm0(session_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(session_secret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.dcapi_submit_response(this.__wbg_ptr, ptr0, len0, ptr1, len1, response);
        return ret;
    }
}
module.exports.DcApi = DcApi;

const JsDcApiSessionDriverFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsdcapisessiondriver_free(ptr >>> 0, 1));

class JsDcApiSessionDriver {

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsDcApiSessionDriverFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsdcapisessiondriver_free(ptr, 0);
    }
}
module.exports.JsDcApiSessionDriver = JsDcApiSessionDriver;

const JsOid4VpSessionStoreFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_jsoid4vpsessionstore_free(ptr >>> 0, 1));
/**
 * WebAssembly-compatible session store that delegates to JavaScript storage implementations.
 *
 * This allows the session store to use any JavaScript storage backend (localStorage,
 * IndexedDB, external databases, etc.) by implementing the required methods in JavaScript.
 */
class JsOid4VpSessionStore {

    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(JsOid4VpSessionStore.prototype);
        obj.__wbg_ptr = ptr;
        JsOid4VpSessionStoreFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        JsOid4VpSessionStoreFinalization.unregister(this);
        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_jsoid4vpsessionstore_free(ptr, 0);
    }
    /**
     * Creates a new WebAssembly session store with JavaScript storage implementation.
     *
     * # Parameters
     *
     * * `store` - JavaScript object implementing the Oid4VpSessionStore interface
     *
     * # Example JavaScript Usage
     *
     * ```javascript
     * class MySessionStore {
     *   async initiate(session) {
     *     // Store session in your preferred storage
     *     localStorage.setItem(`session_${session.uuid}`, JSON.stringify(session));
     *   }
     *
     *   async updateStatus(uuid, status) {
     *     // Update session status
     *     const session = JSON.parse(localStorage.getItem(`session_${uuid}`));
     *     session.status = status;
     *     localStorage.setItem(`session_${uuid}`, JSON.stringify(session));
     *   }
     *
     *   async getSession(uuid) {
     *     // Get session from storage
     *     const sessionData = localStorage.getItem(`session_${uuid}`);
     *     if (!sessionData) throw new Error('Session not found');
     *     return JSON.parse(sessionData);
     *   }
     *
     *   async removeSession(uuid) {
     *     // Remove session from storage
     *     localStorage.removeItem(`session_${uuid}`);
     *   }
     * }
     *
     * const sessionStore = new WasmOid4VpSession(new MySessionStore());
     * ```
     * @param {Oid4VpSessionStore} store
     */
    constructor(store) {
        const ret = wasm.jsoid4vpsessionstore_new(store);
        this.__wbg_ptr = ret >>> 0;
        JsOid4VpSessionStoreFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Helper function to create a simple in-memory session store for testing purposes.
     *
     * This creates a JavaScript Map-based session store that can be used for development
     * and testing without requiring external storage setup.
     *
     * # Example JavaScript Usage
     *
     * ```javascript
     * import { WasmOid4VpSession } from './pkg/dc_api_wasm.js';
     *
     * const sessionStore = WasmOid4VpSession.createMemoryStore();
     * // Use sessionStore with your DcApi instance
     * ```
     * @returns {JsOid4VpSessionStore}
     */
    static createMemoryStore() {
        const ret = wasm.jsoid4vpsessionstore_createMemoryStore();
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return JsOid4VpSessionStore.__wrap(ret[0]);
    }
    /**
     * Utility functions for session management from JavaScript
     * Create a new UUID for session identification
     * @returns {string}
     */
    static generateSessionUuid() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.jsoid4vpsessionstore_generateSessionUuid();
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Parse a UUID string and validate it
     * @param {string} uuid_str
     * @returns {string}
     */
    static parseUuid(uuid_str) {
        let deferred3_0;
        let deferred3_1;
        try {
            const ptr0 = passStringToWasm0(uuid_str, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.jsoid4vpsessionstore_parseUuid(ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Convert a Session to a JavaScript object
     * @param {any} session
     * @returns {any}
     */
    static sessionToJs(session) {
        const ret = wasm.jsoid4vpsessionstore_sessionToJs(session);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Convert a Status to a JavaScript object
     * @param {any} status
     * @returns {any}
     */
    static statusToJs(status) {
        const ret = wasm.jsoid4vpsessionstore_statusToJs(status);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Helper function to create a Status::SentRequestByReference
     * @returns {any}
     */
    static createStatusSentRequestByReference() {
        const ret = wasm.jsoid4vpsessionstore_createStatusSentRequestByReference();
        return ret;
    }
    /**
     * Helper function to create a Status::SentRequest
     * @returns {any}
     */
    static createStatusSentRequest() {
        const ret = wasm.jsoid4vpsessionstore_createStatusSentRequest();
        return ret;
    }
    /**
     * Helper function to create a Status::ReceivedResponse
     * @returns {any}
     */
    static createStatusReceivedResponse() {
        const ret = wasm.jsoid4vpsessionstore_createStatusReceivedResponse();
        return ret;
    }
    /**
     * Helper function to create a Status::Complete with success outcome
     * @param {any} info
     * @returns {any}
     */
    static createStatusCompleteSuccess(info) {
        const ret = wasm.jsoid4vpsessionstore_createStatusCompleteSuccess(info);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Helper function to create a Status::Complete with failure outcome
     * @param {string} reason
     * @returns {any}
     */
    static createStatusCompleteFailure(reason) {
        const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.jsoid4vpsessionstore_createStatusCompleteFailure(ptr0, len0);
        return ret;
    }
    /**
     * Helper function to create a Status::Complete with error outcome
     * @param {string} cause
     * @returns {any}
     */
    static createStatusCompleteError(cause) {
        const ptr0 = passStringToWasm0(cause, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.jsoid4vpsessionstore_createStatusCompleteError(ptr0, len0);
        return ret;
    }
}
module.exports.JsOid4VpSessionStore = JsOid4VpSessionStore;

module.exports.__wbg_Error_0497d5bdba9362e5 = function(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

module.exports.__wbg_String_8f0eb39a4a4c2f66 = function(arg0, arg1) {
    const ret = String(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbg_buffer_a1a27a0dfa70165d = function(arg0) {
    const ret = arg0.buffer;
    return ret;
};

module.exports.__wbg_call_f2db6205e5c51dc8 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.call(arg1, arg2);
    return ret;
}, arguments) };

module.exports.__wbg_call_fbe8be8bf6436ce5 = function() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments) };

module.exports.__wbg_dcapi_new = function(arg0) {
    const ret = DcApi.__wrap(arg0);
    return ret;
};

module.exports.__wbg_done_4d01f352bade43b7 = function(arg0) {
    const ret = arg0.done;
    return ret;
};

module.exports.__wbg_entries_41651c850143b957 = function(arg0) {
    const ret = Object.entries(arg0);
    return ret;
};

module.exports.__wbg_eval_17f2fea482576acf = function() { return handleError(function (arg0, arg1) {
    const ret = eval(getStringFromWasm0(arg0, arg1));
    return ret;
}, arguments) };

module.exports.__wbg_getRandomValues_38a1ff1ea09f6cc7 = function() { return handleError(function (arg0, arg1) {
    globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
}, arguments) };

module.exports.__wbg_getRandomValues_3c9c0d586e575a16 = function() { return handleError(function (arg0, arg1) {
    globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
}, arguments) };

module.exports.__wbg_getSessionUnauthenticated_d0d218118566ba0f = function() { return handleError(function (arg0, arg1, arg2) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg1;
        deferred0_1 = arg2;
        const ret = arg0.getSessionUnauthenticated(getStringFromWasm0(arg1, arg2));
        return ret;
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

module.exports.__wbg_getSession_c989659cbfe20537 = function() { return handleError(function (arg0, arg1, arg2) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg1;
        deferred0_1 = arg2;
        const ret = arg0.getSession(getStringFromWasm0(arg1, arg2));
        return ret;
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

module.exports.__wbg_getSession_d0377b3e861a1a2e = function() { return handleError(function (arg0, arg1, arg2) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg1;
        deferred0_1 = arg2;
        const ret = arg0.getSession(getStringFromWasm0(arg1, arg2));
        return ret;
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

module.exports.__wbg_getTime_2afe67905d873e92 = function(arg0) {
    const ret = arg0.getTime();
    return ret;
};

module.exports.__wbg_get_92470be87867c2e5 = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments) };

module.exports.__wbg_get_a131a44bd1eb6979 = function(arg0, arg1) {
    const ret = arg0[arg1 >>> 0];
    return ret;
};

module.exports.__wbg_getwithrefkey_1dc361bd10053bfe = function(arg0, arg1) {
    const ret = arg0[arg1];
    return ret;
};

module.exports.__wbg_initiate_9e16e763b9e9e9c5 = function() { return handleError(function (arg0, arg1) {
    const ret = arg0.initiate(arg1);
    return ret;
}, arguments) };

module.exports.__wbg_instanceof_ArrayBuffer_a8b6f580b363f2bc = function(arg0) {
    let result;
    try {
        result = arg0 instanceof ArrayBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

module.exports.__wbg_instanceof_Map_80cc65041c96417a = function(arg0) {
    let result;
    try {
        result = arg0 instanceof Map;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

module.exports.__wbg_instanceof_Uint8Array_ca460677bc155827 = function(arg0) {
    let result;
    try {
        result = arg0 instanceof Uint8Array;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

module.exports.__wbg_isArray_5f090bed72bd4f89 = function(arg0) {
    const ret = Array.isArray(arg0);
    return ret;
};

module.exports.__wbg_isSafeInteger_90d7c4674047d684 = function(arg0) {
    const ret = Number.isSafeInteger(arg0);
    return ret;
};

module.exports.__wbg_iterator_4068add5b2aef7a6 = function() {
    const ret = Symbol.iterator;
    return ret;
};

module.exports.__wbg_length_ab6d22b5ead75c72 = function(arg0) {
    const ret = arg0.length;
    return ret;
};

module.exports.__wbg_length_f00ec12454a5d9fd = function(arg0) {
    const ret = arg0.length;
    return ret;
};

module.exports.__wbg_new0_97314565408dea38 = function() {
    const ret = new Date();
    return ret;
};

module.exports.__wbg_newSession_22ebcf441b6ec322 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg1;
        deferred0_1 = arg2;
        const ret = arg0.newSession(getStringFromWasm0(arg1, arg2), arg3);
        return ret;
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

module.exports.__wbg_new_07b483f72211fd66 = function() {
    const ret = new Object();
    return ret;
};

module.exports.__wbg_new_58353953ad2097cc = function() {
    const ret = new Array();
    return ret;
};

module.exports.__wbg_new_a979b4b45bd55c7f = function() {
    const ret = new Map();
    return ret;
};

module.exports.__wbg_new_e30c39c06edaabf2 = function(arg0, arg1) {
    try {
        var state0 = {a: arg0, b: arg1};
        var cb0 = (arg0, arg1) => {
            const a = state0.a;
            state0.a = 0;
            try {
                return __wbg_adapter_149(a, state0.b, arg0, arg1);
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

module.exports.__wbg_new_e52b3efaaa774f96 = function(arg0) {
    const ret = new Uint8Array(arg0);
    return ret;
};

module.exports.__wbg_newnoargs_ff528e72d35de39a = function(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return ret;
};

module.exports.__wbg_next_8bb824d217961b5d = function(arg0) {
    const ret = arg0.next;
    return ret;
};

module.exports.__wbg_next_e2da48d8fff7439a = function() { return handleError(function (arg0) {
    const ret = arg0.next();
    return ret;
}, arguments) };

module.exports.__wbg_queueMicrotask_46c1df247678729f = function(arg0) {
    queueMicrotask(arg0);
};

module.exports.__wbg_queueMicrotask_8acf3ccb75ed8d11 = function(arg0) {
    const ret = arg0.queueMicrotask;
    return ret;
};

module.exports.__wbg_removeSession_08ae4d19166f9a39 = function() { return handleError(function (arg0, arg1, arg2) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg1;
        deferred0_1 = arg2;
        const ret = arg0.removeSession(getStringFromWasm0(arg1, arg2));
        return ret;
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

module.exports.__wbg_removeSession_4e44b9d7b70df102 = function() { return handleError(function (arg0, arg1, arg2) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg1;
        deferred0_1 = arg2;
        const ret = arg0.removeSession(getStringFromWasm0(arg1, arg2));
        return ret;
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

module.exports.__wbg_resolve_0dac8c580ffd4678 = function(arg0) {
    const ret = Promise.resolve(arg0);
    return ret;
};

module.exports.__wbg_set_3f1d0b984ed272ed = function(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
};

module.exports.__wbg_set_7422acbe992d64ab = function(arg0, arg1, arg2) {
    arg0[arg1 >>> 0] = arg2;
};

module.exports.__wbg_set_d6bdfd275fb8a4ce = function(arg0, arg1, arg2) {
    const ret = arg0.set(arg1, arg2);
    return ret;
};

module.exports.__wbg_set_fe4e79d1ed3b0e9b = function(arg0, arg1, arg2) {
    arg0.set(arg1, arg2 >>> 0);
};

module.exports.__wbg_static_accessor_GLOBAL_487c52c58d65314d = function() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_GLOBAL_THIS_ee9704f328b6b291 = function() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_SELF_78c9e3071b912620 = function() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_static_accessor_WINDOW_a093d21393777366 = function() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

module.exports.__wbg_then_82ab9fb4080f1707 = function(arg0, arg1, arg2) {
    const ret = arg0.then(arg1, arg2);
    return ret;
};

module.exports.__wbg_then_db882932c0c714c6 = function(arg0, arg1) {
    const ret = arg0.then(arg1);
    return ret;
};

module.exports.__wbg_updateSession_85e2307a6e1e7a87 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg1;
        deferred0_1 = arg2;
        const ret = arg0.updateSession(getStringFromWasm0(arg1, arg2), arg3);
        return ret;
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

module.exports.__wbg_updateStatus_1828c280d679f70b = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg1;
        deferred0_1 = arg2;
        const ret = arg0.updateStatus(getStringFromWasm0(arg1, arg2), arg3);
        return ret;
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
}, arguments) };

module.exports.__wbg_value_17b896954e14f896 = function(arg0) {
    const ret = arg0.value;
    return ret;
};

module.exports.__wbindgen_as_number = function(arg0) {
    const ret = +arg0;
    return ret;
};

module.exports.__wbindgen_bigint_from_i64 = function(arg0) {
    const ret = arg0;
    return ret;
};

module.exports.__wbindgen_bigint_from_u64 = function(arg0) {
    const ret = BigInt.asUintN(64, arg0);
    return ret;
};

module.exports.__wbindgen_bigint_get_as_i64 = function(arg0, arg1) {
    const v = arg1;
    const ret = typeof(v) === 'bigint' ? v : undefined;
    getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

module.exports.__wbindgen_boolean_get = function(arg0) {
    const v = arg0;
    const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
    return ret;
};

module.exports.__wbindgen_cb_drop = function(arg0) {
    const obj = arg0.original;
    if (obj.cnt-- == 1) {
        obj.a = 0;
        return true;
    }
    const ret = false;
    return ret;
};

module.exports.__wbindgen_closure_wrapper4182 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 1629, __wbg_adapter_52);
    return ret;
};

module.exports.__wbindgen_debug_string = function(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbindgen_in = function(arg0, arg1) {
    const ret = arg0 in arg1;
    return ret;
};

module.exports.__wbindgen_init_externref_table = function() {
    const table = wasm.__wbindgen_export_4;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

module.exports.__wbindgen_is_bigint = function(arg0) {
    const ret = typeof(arg0) === 'bigint';
    return ret;
};

module.exports.__wbindgen_is_function = function(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

module.exports.__wbindgen_is_null = function(arg0) {
    const ret = arg0 === null;
    return ret;
};

module.exports.__wbindgen_is_object = function(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

module.exports.__wbindgen_is_string = function(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
};

module.exports.__wbindgen_is_undefined = function(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

module.exports.__wbindgen_jsval_eq = function(arg0, arg1) {
    const ret = arg0 === arg1;
    return ret;
};

module.exports.__wbindgen_jsval_loose_eq = function(arg0, arg1) {
    const ret = arg0 == arg1;
    return ret;
};

module.exports.__wbindgen_memory = function() {
    const ret = wasm.memory;
    return ret;
};

module.exports.__wbindgen_number_get = function(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

module.exports.__wbindgen_number_new = function(arg0) {
    const ret = arg0;
    return ret;
};

module.exports.__wbindgen_string_get = function(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbindgen_string_new = function(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

const path = require('path').join(__dirname, 'dc_api_wasm_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

wasm.__wbindgen_start();

