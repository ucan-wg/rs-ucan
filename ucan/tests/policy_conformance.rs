//! Tests for policy predicate conformance using fixtures.
mod policy_conformance {
    use std::sync::OnceLock;

    use ipld_core::ipld::Ipld;
    use testresult::TestResult;
    use ucan::delegation::policy::predicate::Predicate;

    type SetupResult = Result<(Ipld, Vec<Vec<Predicate>>), Box<dyn std::error::Error>>;

    const POLICY_FIXTURE_STR: &str = include_str!("./fixtures/policy.json");
    static POLICY_FIXTURE: OnceLock<serde_json::Value> = OnceLock::new();
    fn policy_fixture() -> &'static serde_json::Value {
        #[allow(clippy::expect_used)]
        POLICY_FIXTURE.get_or_init(|| {
            serde_json::from_str(POLICY_FIXTURE_STR).expect("fixture is invalid JSON")
        })
    }

    mod valid {
        use super::*;

        fn setup(idx: usize) -> SetupResult {
            let json: serde_json::Value = serde_json::from_str(POLICY_FIXTURE_STR)?;

            let args_value = json
                .get("valid")
                .and_then(|v| v.get(idx))
                .and_then(|v| v.get("args"))
                .ok_or("fixture missing valid/idx/args")?;
            let args_bytes = serde_json::to_vec(args_value)?;
            let args_ipld: Ipld = serde_ipld_dagjson::from_slice(&args_bytes)?;

            let policy_json = policy_fixture()
                .get("valid")
                .and_then(|v| v.get(idx))
                .and_then(|v| v.get("policies"))
                .ok_or("fixture missing valid/idx/policies")?;
            let policy_bytes = serde_json::to_vec(policy_json)?;
            let policy_ipld: Ipld = serde_ipld_dagjson::from_slice(&policy_bytes)?;

            let mut policies = Vec::new();
            match policy_ipld {
                Ipld::List(ipld_policies) => {
                    for ipld_policy in ipld_policies {
                        let mut policy: Vec<Predicate> = Vec::new();
                        match ipld_policy {
                            Ipld::List(ipld_predicates) => {
                                for ipld_predicate in &ipld_predicates {
                                    let predicate = Predicate::try_from(ipld_predicate.clone())?;
                                    policy.push(predicate);
                                }
                            }
                            Ipld::Null
                            | Ipld::Bool(_)
                            | Ipld::Integer(_)
                            | Ipld::Float(_)
                            | Ipld::String(_)
                            | Ipld::Bytes(_)
                            | Ipld::Map(_)
                            | Ipld::Link(_) => return Err("expected policy to be a list".into()),
                        }
                        policies.push(policy);
                    }
                }
                Ipld::Null
                | Ipld::Bool(_)
                | Ipld::Integer(_)
                | Ipld::Float(_)
                | Ipld::String(_)
                | Ipld::Bytes(_)
                | Ipld::Map(_)
                | Ipld::Link(_) => return Err("expected policies to be a list".into()),
            }

            Ok((args_ipld, policies))
        }

        mod scenario_zero {
            use super::*;

            static SCENARIO_ZERO_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_zero_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_ZERO_FIXTURE.get_or_init(|| setup(0).expect("scenario 0 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_first_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(1).ok_or("missing policy 1")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_second_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(2).ok_or("missing policy 2")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_third_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(3).ok_or("missing policy 3")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_fourth_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(4).ok_or("missing policy 4")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_fifth_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(5).ok_or("missing policy 5")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }
        }

        mod scenario_one {
            use super::*;

            static SCENARIO_ONE_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_one_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_ONE_FIXTURE.get_or_init(|| setup(1).expect("scenario 1 setup failed"))
            }

            #[test]
            fn test_the_lone_policy() -> TestResult {
                let (args, policies) = scenario_one_fixture();

                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(policy.clone().run(args)?);
                }

                Ok(())
            }
        }

        mod scenario_two {
            use super::*;

            static SCENARIO_TWO_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_two_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_TWO_FIXTURE.get_or_init(|| setup(2).expect("scenario 2 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_two_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_first_policy() -> TestResult {
                let (args, policies) = scenario_two_fixture();
                for policy in policies.get(1).ok_or("missing policy 1")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_second_policy() -> TestResult {
                let (args, policies) = scenario_two_fixture();
                for policy in policies.get(2).ok_or("missing policy 2")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_third_policy() -> TestResult {
                let (args, policies) = scenario_two_fixture();
                for policy in policies.get(3).ok_or("missing policy 3")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_fourth_policy() -> TestResult {
                let (args, policies) = scenario_two_fixture();
                for policy in policies.get(4).ok_or("missing policy 4")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_fifth_policy() -> TestResult {
                let (args, policies) = scenario_two_fixture();
                for policy in policies.get(5).ok_or("missing policy 5")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_sixth_policy() -> TestResult {
                let (args, policies) = scenario_two_fixture();
                for policy in policies.get(6).ok_or("missing policy 6")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }
        }

        mod scenario_three {
            use super::*;

            static SCENARIO_THREE_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_three_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_THREE_FIXTURE.get_or_init(|| setup(3).expect("scenario 3 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_three_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }
        }

        mod scenario_four {
            use super::*;

            static SCENARIO_FOUR_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_four_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_FOUR_FIXTURE.get_or_init(|| setup(4).expect("scenario 4 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_four_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }
        }

        mod scenario_five {
            use super::*;

            static SCENARIO_FIVE_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_five_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_FIVE_FIXTURE.get_or_init(|| setup(5).expect("scenario 5 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_five_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(policy.clone().run(args)?);
                }
                Ok(())
            }
        }
    }

    mod invalid {
        use super::*;

        fn setup(idx: usize) -> SetupResult {
            let json: serde_json::Value = serde_json::from_str(POLICY_FIXTURE_STR)?;

            let args_value = json
                .get("invalid")
                .and_then(|v| v.get(idx))
                .and_then(|v| v.get("args"))
                .ok_or("fixture missing invalid/idx/args")?;
            let args_bytes = serde_json::to_vec(args_value)?;
            let args_ipld: Ipld = serde_ipld_dagjson::from_slice(&args_bytes)?;

            let policy_json = policy_fixture()
                .get("invalid")
                .and_then(|v| v.get(idx))
                .and_then(|v| v.get("policies"))
                .ok_or("fixture missing invalid/idx/policies")?;
            let policy_bytes = serde_json::to_vec(policy_json)?;
            let policy_ipld: Ipld = serde_ipld_dagjson::from_slice(&policy_bytes)?;

            let mut policies = Vec::new();
            match policy_ipld {
                Ipld::List(ipld_policies) => {
                    for ipld_policy in ipld_policies {
                        let mut policy: Vec<Predicate> = Vec::new();
                        match ipld_policy {
                            Ipld::List(ipld_predicates) => {
                                for ipld_predicate in &ipld_predicates {
                                    let predicate = Predicate::try_from(ipld_predicate.clone())?;
                                    policy.push(predicate);
                                }
                            }
                            Ipld::Null
                            | Ipld::Bool(_)
                            | Ipld::Integer(_)
                            | Ipld::Float(_)
                            | Ipld::String(_)
                            | Ipld::Bytes(_)
                            | Ipld::Map(_)
                            | Ipld::Link(_) => return Err("expected policy to be a list".into()),
                        }
                        policies.push(policy);
                    }
                }
                Ipld::Null
                | Ipld::Bool(_)
                | Ipld::Integer(_)
                | Ipld::Float(_)
                | Ipld::String(_)
                | Ipld::Bytes(_)
                | Ipld::Map(_)
                | Ipld::Link(_) => return Err("expected policies to be a list".into()),
            }

            Ok((args_ipld, policies))
        }

        mod scenario_zero {
            use super::*;

            static SCENARIO_ZERO_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_zero_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_ZERO_FIXTURE.get_or_init(|| setup(0).expect("scenario 0 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(!policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_first_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(1).ok_or("missing policy 1")? {
                    assert!(!policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_second_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(2).ok_or("missing policy 2")? {
                    assert!(!policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_third_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(3).ok_or("missing policy 3")? {
                    assert!(!policy.clone().run(args)?);
                }
                Ok(())
            }

            #[test]
            fn test_fourth_policy() -> TestResult {
                let (args, policies) = scenario_zero_fixture();
                for policy in policies.get(4).ok_or("missing policy 4")? {
                    assert!(!policy.clone().run(args)?);
                }
                Ok(())
            }
        }

        mod scenario_one {
            use super::*;

            static SCENARIO_ONE_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_one_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_ONE_FIXTURE.get_or_init(|| setup(0).expect("scenario 1 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_one_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(!policy.clone().run(args)?);
                }
                Ok(())
            }
        }

        mod scenario_two {
            use super::*;

            static SCENARIO_TWO_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_two_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_TWO_FIXTURE.get_or_init(|| setup(0).expect("scenario 2 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_two_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(!policy.clone().run(args)?);
                }
                Ok(())
            }
        }

        mod scenario_three {
            use super::*;

            static SCENARIO_THREE_FIXTURE: OnceLock<(Ipld, Vec<Vec<Predicate>>)> = OnceLock::new();
            fn scenario_three_fixture() -> &'static (Ipld, Vec<Vec<Predicate>>) {
                #[allow(clippy::expect_used)]
                SCENARIO_THREE_FIXTURE.get_or_init(|| setup(0).expect("scenario 3 setup failed"))
            }

            #[test]
            fn test_zeroth_policy() -> TestResult {
                let (args, policies) = scenario_three_fixture();
                for policy in policies.first().ok_or("missing policy 0")? {
                    assert!(!policy.clone().run(args)?);
                }
                Ok(())
            }
        }
    }
}
