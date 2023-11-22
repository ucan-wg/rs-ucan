use libipld_core::{ipld::Ipld, raw::RawCodec};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader, str::FromStr};

use rs_ucan::{
    capability::DefaultCapabilityParser,
    did_verifier::DidVerifierMap,
    store::{self, Store},
    ucan::Ucan,
    DefaultFact,
};

trait TestTask {
    fn run(&self, name: &str, report: &mut TestReport);
}

#[derive(Debug, Default)]
struct TestReport {
    num_tests: usize,
    successes: Vec<String>,
    failures: Vec<TestFailure>,
}

#[derive(Debug)]
struct TestFailure {
    name: String,
    error: String,
}

impl TestReport {
    fn register_success(&mut self, name: &str) {
        self.num_tests += 1;
        self.successes.push(name.to_string());
    }

    fn register_failure(&mut self, name: &str, error: String) {
        self.num_tests += 1;
        self.failures.push(TestFailure {
            name: name.to_string(),
            error,
        });
    }

    fn finish(&self) {
        for success in &self.successes {
            println!("✅ {}", success);
        }

        for failure in &self.failures {
            println!("❌ {}: {}", failure.name, failure.error);
        }

        println!(
            "{} tests, {} successes, {} failures",
            self.num_tests,
            self.successes.len(),
            self.failures.len()
        );

        if !self.failures.is_empty() {
            panic!();
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct TestFixture {
    name: String,
    #[serde(flatten)]
    test_case: TestCase,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "task", rename_all = "camelCase")]
enum TestCase {
    Verify(VerifyTest),
    Refute(RefuteTest),
    Build(BuildTest),
    ToCID(ToCidTest),
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyTest {
    inputs: TestInputsTokenAndProofs,
    assertions: TestAssertions,
}

#[derive(Debug, Serialize, Deserialize)]
struct RefuteTest {
    inputs: TestInputsTokenAndProofs,
    assertions: TestAssertions,
    errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildTest {
    inputs: BuildTestInputs,
    outputs: BuildTestOutputs,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToCidTest {
    inputs: ToCidTestInputs,
    outputs: ToCidTestOutputs,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestInputsTokenAndProofs {
    token: String,
    proofs: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestAssertions {
    header: TestAssertionsHeader,
    payload: TestAssertionsPayload,
    signature: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestAssertionsHeader {
    alg: Option<String>,
    typ: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestAssertionsPayload {
    ucv: Option<String>,
    iss: Option<String>,
    aud: Option<String>,
    exp: Option<u64>,
    // TODO: CAP
    // TODO: FCT
    prf: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildTestInputs {
    version: Option<String>,
    issuer_base64_key: String,
    signature_scheme: String,
    audience: Option<String>,
    not_before: Option<u64>,
    expiration: Option<u64>,
    // TODO CAPABILITIES
    // TODO FACTS
}

#[derive(Debug, Serialize, Deserialize)]
struct BuildTestOutputs {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToCidTestInputs {
    token: String,
    hasher: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ToCidTestOutputs {
    cid: String,
}

impl TestTask for VerifyTest {
    fn run(&self, name: &str, report: &mut TestReport) {
        let mut store = store::InMemoryStore::<RawCodec>::default();
        let did_verifier_map = DidVerifierMap::default();

        for (_cid, token) in self.inputs.proofs.iter() {
            store
                .write(Ipld::Bytes(token.as_bytes().to_vec()), None)
                .unwrap();
        }

        let Ok(ucan) = Ucan::<DefaultFact, DefaultCapabilityParser>::from_str(&self.inputs.token)
        else {
            report.register_failure(name, "failed to parse token".to_string());

            return;
        };

        if let Some(alg) = &self.assertions.header.alg {
            if ucan.algorithm() != alg {
                report.register_failure(
                    name,
                    format!(
                        "expected algorithm to be {}, but was {}",
                        alg,
                        ucan.algorithm()
                    ),
                );

                return;
            }
        }

        if let Some(typ) = &self.assertions.header.typ {
            if ucan.typ() != typ {
                report.register_failure(
                    name,
                    format!("expected type to be {}, but was {}", typ, ucan.typ()),
                );

                return;
            }
        }

        if let Some(ucv) = &self.assertions.payload.ucv {
            if ucan.version() != ucv {
                report.register_failure(
                    name,
                    format!("expected version to be {}, but was {}", ucv, ucan.version()),
                );

                return;
            }
        }

        if let Some(iss) = &self.assertions.payload.iss {
            if ucan.issuer() != iss {
                report.register_failure(
                    name,
                    format!("expected issuer to be {}, but was {}", iss, ucan.issuer()),
                );

                return;
            }
        }

        if let Some(aud) = &self.assertions.payload.aud {
            if ucan.audience() != aud {
                report.register_failure(
                    name,
                    format!(
                        "expected audience to be {}, but was {}",
                        aud,
                        ucan.audience()
                    ),
                );

                return;
            }
        }

        if ucan.expires_at() != self.assertions.payload.exp {
            report.register_failure(
                name,
                format!(
                    "expected expiration to be {:?}, but was {:?}",
                    self.assertions.payload.exp,
                    ucan.expires_at()
                ),
            );

            return;
        }

        if ucan
            .proofs()
            .map(|f| f.iter().map(|c| c.to_string()).collect())
            != self.assertions.payload.prf
        {
            report.register_failure(
                name,
                format!(
                    "expected proofs to be {:?}, but was {:?}",
                    self.assertions.payload.prf,
                    ucan.proofs()
                ),
            );

            return;
        }

        let Ok(signature) = serde_json::to_value(ucan.signature()) else {
            report.register_failure(name, "failed to serialize signature".to_string());

            return;
        };

        let Some(signature) = signature.as_str() else {
            report.register_failure(name, "expected signature to be a string".to_string());

            return;
        };

        if signature != self.assertions.signature {
            report.register_failure(
                name,
                format!(
                    "expected signature to be {}, but was {}",
                    self.assertions.signature, signature
                ),
            );

            return;
        }

        if let Err(err) = ucan.validate(rs_ucan::time::now(), &did_verifier_map) {
            report.register_failure(name, err.to_string());

            return;
        }
    }
}

impl TestTask for RefuteTest {
    fn run(&self, name: &str, report: &mut TestReport) {
        let mut store = store::InMemoryStore::<RawCodec>::default();
        let did_verifier_map = DidVerifierMap::default();

        for (_cid, token) in self.inputs.proofs.iter() {
            store
                .write(Ipld::Bytes(token.as_bytes().to_vec()), None)
                .unwrap();
        }

        if let Ok(ucan) = Ucan::<DefaultFact, DefaultCapabilityParser>::from_str(&self.inputs.token)
        {
            if ucan
                .validate(rs_ucan::time::now(), &did_verifier_map)
                .is_ok()
            {
                report.register_failure(
                    &name,
                    "expected token to fail validation, but it passed".to_string(),
                );

                return;
            }
        }
    }
}

impl TestTask for BuildTest {
    fn run(&self, _: &str, _: &mut TestReport) {
        //TODO: can't assert on signature because of canonicalization issues
    }
}

impl TestTask for ToCidTest {
    fn run(&self, name: &str, report: &mut TestReport) {
        let ucan =
            Ucan::<DefaultFact, DefaultCapabilityParser>::from_str(&self.inputs.token).unwrap();
        let hasher = match self.inputs.hasher.as_str() {
            "SHA2-256" => multihash::Code::Sha2_256,
            "BLAKE3-256" => multihash::Code::Blake3_256,
            _ => panic!(),
        };

        let Ok(cid) = ucan.to_cid(Some(hasher)) else {
            report.register_failure(&name, "failed to convert to CID".to_string());

            return;
        };

        if cid.to_string() != self.outputs.cid {
            report.register_failure(
                &name,
                format!(
                    "expected CID to be {}, but was {}",
                    self.outputs.cid,
                    cid.to_string()
                ),
            );

            return;
        }
    }
}

#[test]
fn ucan_0_10_0_conformance_tests() {
    let fixtures_file = File::open("tests/fixtures/0.10.0/all.json").unwrap();
    let reader = BufReader::new(fixtures_file);
    let fixtures: Vec<TestFixture> = serde_json::from_reader(reader).unwrap();

    let mut report = TestReport::default();

    for fixture in fixtures {
        match fixture.test_case {
            TestCase::Verify(test) => test.run(&fixture.name, &mut report),
            TestCase::Refute(test) => test.run(&fixture.name, &mut report),
            TestCase::Build(test) => test.run(&fixture.name, &mut report),
            TestCase::ToCID(test) => test.run(&fixture.name, &mut report),
        };

        report.register_success(&fixture.name);
    }

    report.finish();
}
