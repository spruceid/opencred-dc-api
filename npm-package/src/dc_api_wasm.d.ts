/* tslint:disable */
/* eslint-disable */
export class DcApi {
  private constructor();
  free(): void;
  static new(key: string, base_url: string, submission_endpoint: string, reference_endpoint: string, cert_chain_pem: Uint8Array, oid4vp_session_store: JsOid4VpSessionStore, js_dc_api_session_store: DcApiSessionStore): Promise<DcApi>;
  create_new_session(): Promise<any>;
  initiate_request(session_id: string, session_secret: string, request: any, user_agent?: string | null): Promise<any>;
  submit_response(session_id: string, session_secret: string, response: any): Promise<any>;
}
export class JsDcApiSessionDriver {
  private constructor();
  free(): void;
}
/**
 * WebAssembly-compatible session store that delegates to JavaScript storage implementations.
 *
 * This allows the session store to use any JavaScript storage backend (localStorage,
 * IndexedDB, external databases, etc.) by implementing the required methods in JavaScript.
 */
export class JsOid4VpSessionStore {
  free(): void;
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
   */
  constructor(store: Oid4VpSessionStore);
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
   */
  static createMemoryStore(): JsOid4VpSessionStore;
  /**
   * Utility functions for session management from JavaScript
   * Create a new UUID for session identification
   */
  static generateSessionUuid(): string;
  /**
   * Parse a UUID string and validate it
   */
  static parseUuid(uuid_str: string): string;
  /**
   * Convert a Session to a JavaScript object
   */
  static sessionToJs(session: any): any;
  /**
   * Convert a Status to a JavaScript object
   */
  static statusToJs(status: any): any;
  /**
   * Helper function to create a Status::SentRequestByReference
   */
  static createStatusSentRequestByReference(): any;
  /**
   * Helper function to create a Status::SentRequest
   */
  static createStatusSentRequest(): any;
  /**
   * Helper function to create a Status::ReceivedResponse
   */
  static createStatusReceivedResponse(): any;
  /**
   * Helper function to create a Status::Complete with success outcome
   */
  static createStatusCompleteSuccess(info: any): any;
  /**
   * Helper function to create a Status::Complete with failure outcome
   */
  static createStatusCompleteFailure(reason: string): any;
  /**
   * Helper function to create a Status::Complete with error outcome
   */
  static createStatusCompleteError(cause: string): any;
}
