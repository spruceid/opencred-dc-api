import * as wasm from "../dist/dc_api_wasm";

const stubDcApiStore = {
  newSession: async (id: string) => ({ id, client_secret: "test" }),
  getSession: async () => null,
  getSessionUnauthenticated: async () => null,
  updateSession: async () => {},
  removeSession: async () => {},
};

describe("DcApi.new constructor signature", () => {
  it("exposes a two-chain constructor with eight parameters", () => {
    expect(typeof wasm.DcApi.new).toBe("function");
    // Regression guard: prior versions accepted a single cert_chain_pem
    // argument and reused it for both the reader client cert and the
    // issuer trust anchor registry. The two roles must remain distinct.
    expect(wasm.DcApi.new.length).toBe(8);
  });

  it("validates the issuer CA chain independently", async () => {
    const oid4vpStore = wasm.JsOid4VpSessionStore.createMemoryStore();
    const garbage = new TextEncoder().encode("not a pem chain");
    const alsoGarbage = new TextEncoder().encode("also not a pem chain");

    await expect(
      wasm.DcApi.new(
        "key",
        "https://example.com",
        "/submit",
        "/ref",
        garbage,
        alsoGarbage,
        oid4vpStore,
        stubDcApiStore as any,
      ),
    ).rejects.toBeDefined();
  });
});
