use anyhow::{anyhow, Context};
use rstest::{fixture, rstest};
use solatic::dimacs_parser;
use solatic::LiteralChoice;
use solatic::SolverArgs;
use std::fs;
use std::sync::LazyLock;

#[derive(Debug, Clone)]
pub struct TestConfig {
    pub is_verbose: bool,
    pub should_update_expect: bool,
    pub input_path: &'static str,
    pub expected_result_path: &'static str,
}

static CONFIG: LazyLock<TestConfig> = LazyLock::new(|| TestConfig {
    is_verbose: std::env::var("VERBOSE").unwrap_or_default() == "1",
    should_update_expect: std::env::var("UPDATE_EXPECT").unwrap_or_default() == "1",
    input_path: "tests/fixtures/inputs",
    expected_result_path: "tests/fixtures/expected",
});

#[fixture]
fn config() -> &'static TestConfig {
    &CONFIG
}

#[fixture]
fn literal_choice() -> LiteralChoice {
    LiteralChoice::False
}

#[rstest]
#[case::sat_3_vars("SAT-3-vars")]
#[case::unsat_4_2_bit_all_diff("UNSAT-4-2-bit-all-diff")]
#[case::unsat_5_2_bit_all_diff("UNSAT-5-2-bit-all-diff")]
#[case::unsat_8_3_bit_all_diff("UNSAT-8-3-bit-all-diff")]
#[case::unsat_9_3_bit_all_diff("UNSAT-9-8-bit-all-diff-one-hot")]
#[case::unsat_10_9_bit_all_diff("UNSAT-10-9-bit-all-diff-one-hot")]
fn test_all_cfg(config: &TestConfig, #[case] test_stem: &str) -> anyhow::Result<()> {
    run_cfg(config, test_stem, test_stem, LiteralChoice::False)?;
    run_cfg(config, test_stem, test_stem, LiteralChoice::True)
}

#[rstest]
#[case::sat_4_2_bit_mostly_all_diff("SAT-4-2-bit-mostly-all-diff")]
#[case::sat_10_9_mostly_bit_all_diff("SAT-10-9-bit-mostly-all-diff-one-hot")]
#[case::unsat_12_11_bit_all_one_hot("UNSAT-12-11-bit-all-diff-one-hot")]
fn test_false_cfg(config: &TestConfig, #[case] test_stem: &str) -> anyhow::Result<()> {
    run_cfg(config, test_stem, test_stem, LiteralChoice::False)
}

#[rstest]
#[case::sat_4_2_bit_mostly_all_diff("SAT-4-2-bit-mostly-all-diff")]
#[case::sat_10_9_mostly_bit_all_diff("SAT-10-9-bit-mostly-all-diff-one-hot")]
// too slow: UNSAT-12-11-bit-all-diff-one-hot
fn test_true_cfg(config: &TestConfig, #[case] test_stem: &str) -> anyhow::Result<()> {
    let true_stem = format!("{test_stem}-true");
    run_cfg(config, test_stem, &true_stem, LiteralChoice::True)
}

fn run_cfg(
    config: &TestConfig,
    test_stem: &str,
    expected_stem: &str,
    choose_literal: LiteralChoice,
) -> anyhow::Result<()> {
    let test_path = format!("{}/{test_stem}.cnf", config.input_path);
    let input_cnf = fs::read_to_string(&test_path)?;
    let expected_result_path = format!("{}/{expected_stem}.expected", config.expected_result_path);
    if config.is_verbose {
        println!("CNF {expected_result_path}:");
        println!("{input_cnf}");
    }
    let mut solver = dimacs_parser::from_reader(
        &SolverArgs {
            choose_literal,
            ..Default::default()
        },
        input_cnf.as_bytes(),
    )
    .with_context(|| format!("{test_path}: in dimacs parser:"))?;
    let mut result_buffer: Vec<u8> = Vec::new();
    solver.solve_and_write(&mut result_buffer)?;
    if config.is_verbose {
        println!("{}", String::from_utf8_lossy(&result_buffer));
    }
    if config.should_update_expect {
        std::fs::write(expected_result_path, &result_buffer)?;
    } else {
        let expected_str = std::fs::read_to_string(&expected_result_path)
            .with_context(|| format!("While opening {expected_result_path}:"))?;
        if expected_str.as_bytes() != result_buffer {
            return Err(anyhow!("{test_stem}: mismatch expected results"));
        }
    }

    Ok(())
}
