import * as wasm from "../dist/dc_api_wasm";

const stubDcApiStore = {
  newSession: async (id: string) => ({ id, client_secret: "test" }),
  getSession: async () => null,
  getSessionUnauthenticated: async () => null,
  updateSession: async () => {},
  removeSession: async () => {},
};

describe("DcApi.new typed config", () => {
  it("accepts a DcApiConfig object as the first argument", () => {
    expect(typeof wasm.DcApi.new).toBe("function");
    // Regression guard: prior versions accepted a single cert_chain_pem
    // argument and reused it for both reader and issuer trust roles.
    // The typed config object now keeps the two chains distinct.
    expect(wasm.DcApi.new.length).toBe(3);
  });

  it("validates the issuer CA chain when given garbage PEM", async () => {
    const oid4vpStore = wasm.JsOid4VpSessionStore.createMemoryStore();

    const config: wasm.DcApiConfig = {
      key: "key",
      baseUrl: "https://example.com",
      submissionEndpoint: "/submit",
      referenceEndpoint: "/ref",
      issuerCaX5cPem: new TextEncoder().encode("not a pem chain"),
      readerCaX5cPem: new TextEncoder().encode("also not a pem chain"),
    };

    await expect(
      wasm.DcApi.new(config, oid4vpStore, stubDcApiStore as any),
    ).rejects.toBeDefined();
  });
});
