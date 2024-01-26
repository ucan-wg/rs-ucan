import assert from "assert";
import { build, decode } from "../dist/bundler/ucan.js";

describe("decode", async function () {
  let ucan = await decode(
    "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhdWQiOiJkaWQ6a2V5Ono2TWtmZkRaQ2tDVFdyZWc4ODY4ZkcxRkdGb2djSmo1WDZQWTkzcFBjV0RuOWJvYiIsImNhcCI6e30sImV4cCI6OTI0NjIxMTIwMCwiaXNzIjoiZGlkOmtleTp6Nk1razg5YkMzSnJWcUtpZTcxWUVjYzVNMVNNVnh1Q2dOeDZ6TFo4U1lKc3hBTGkiLCJ1Y3YiOiIwLjEwLjAifQ.pkJxQke-FDVB1Eg_7Jh2socNBKgo6_0OF1XXRfRMazmpXBG37tScYGAzJKB2Z4RFvSBpbBu29Sozrv4GQLFrDg",
  );

  it("decodes the signature", async function () {
    let actual = ucan.signature;
    let expected = new Uint8Array([
      166, 66, 113, 66, 71, 190, 20, 53, 65, 212, 72, 63, 236, 152, 118, 178,
      135, 13, 4, 168, 40, 235, 253, 14, 23, 85, 215, 69, 244, 76, 107, 57, 169,
      92, 17, 183, 238, 212, 156, 96, 96, 51, 36, 160, 118, 103, 132, 69, 189,
      32, 105, 108, 27, 182, 245, 42, 51, 174, 254, 6, 64, 177, 107, 14,
    ]);

    assert.equal(actual.byteLength, expected.byteLength);
    assert.ok(actual.every((v, i) => v === expected[i]));
  });

  it("decodes the typ", async function () {
    let actual = ucan.typ;
    let expected = "JWT";

    assert.equal(actual, expected);
  });

  it("decodes the alg", async function () {
    let actual = ucan.algorithm;
    let expected = "EdDSA";

    assert.equal(actual, expected);
  });

  it("decodes the iss", async function () {
    let actual = ucan.issuer;
    let expected = "did:key:z6Mkk89bC3JrVqKie71YEcc5M1SMVxuCgNx6zLZ8SYJsxALi";

    assert.equal(actual, expected);
  });

  it("decodes the aud", async function () {
    let actual = ucan.audience;
    let expected = "did:key:z6MkffDZCkCTWreg8868fG1FGFogcJj5X6PY93pPcWDn9bob";

    assert.equal(actual, expected);
  });

  it("decodes the exp", async function () {
    let actual = ucan.expiresAt.getTime();
    let expected = new Date(9246211200 * 1000).getTime();

    assert.equal(actual, expected);
  });

  it("decodes the nbf", async function () {
    let actual = ucan.notBefore;
    let expected = null;

    assert.equal(actual, expected);
  });

  it("decodes the nnc", async function () {
    let actual = ucan.nonce;
    let expected = null;

    assert.equal(actual, expected);
  });

  it("decodes the facts", async function () {
    let actual = ucan.facts;
    let expected = null;

    assert.equal(actual, expected);
  });

  it("decodes the ucn", async function () {
    let actual = ucan.version;
    let expected = "0.10.0";

    assert.equal(actual, expected);
  });

  it("preserves the CID", async function () {
    let actual = ucan.cid();
    let expected =
      "bafkreifmws7u5w6nluprxu5zcsun2wcp7rioxjy2qem6pjj5z367dp64li";

    assert.equal(actual, expected);
  });
});

describe("build", async function () {
  const RSA_ALG = "RSASSA-PKCS1-v1_5";
  const DEFAULT_KEY_SIZE = 2048;
  const DEFAULT_HASH_ALG = "SHA-256";
  const SALT_LEGNTH = 128;

  it("builds the ucan", async function () {
    let keypair = await crypto.subtle.generateKey(
      {
        name: RSA_ALG,
        modulusLength: DEFAULT_KEY_SIZE,
        publicExponent: new Uint8Array([0x01, 0x00, 0x01]),
        hash: { name: DEFAULT_HASH_ALG },
      },
      false,
      ["sign", "verify"],
    );

    let ucan = await build(keypair, "did:key:test", {});

    assert.equal(ucan.signature, null);
  });
});
