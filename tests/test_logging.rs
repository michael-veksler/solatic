use indoc::indoc;
use solatic::{dprint, dprint_once, dprintln, dprintln_once};
use std::process::Command;

#[test]
fn dprint_dprintln_output() {
    for i in 0..=4 {
        dprintln!(i % 2 == 0, "{index}: named placeholder", index = i);
        dprintln!(i % 2 == 0, "{i}: inplace");
        dprint!(i % 2 == 0, "{}: ", i);
        dprintln!(i % 2 == 0, "unnamed placeholder");
    }
}

#[test]
fn dprint_dprintln_validate() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "dprint_dprintln_output", "--nocapture"])
        .output()
        .unwrap();

    assert!(output.status.success(), "dprintln_output child failed");
    let output_str = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        &output_str,
        indoc! {"
                  0: named placeholder
                  0: inplace
                  0: unnamed placeholder
                  2: named placeholder
                  2: inplace
                  2: unnamed placeholder
                  4: named placeholder
                  4: inplace
                  4: unnamed placeholder
                  "}
    );
}

#[test]
fn dprint_dprintln_once_output() {
    for i in 0..=4 {
        dprintln_once!(i % 2 == 0, "{index}: named placeholder", index = i);
        dprintln_once!(i % 2 == 0, "{i}: inplace");
        dprint_once!(i % 2 == 0, "{}: ", i);
        dprintln_once!(i % 2 == 0, "unnamed placeholder");
    }
}

#[test]
fn dprint_dprintln_once_validate() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "dprint_dprintln_once_output", "--nocapture"])
        .output()
        .unwrap();

    assert!(output.status.success(), "dprintln_output child failed");
    let output_str = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        &output_str,
        indoc! {"
                  0: named placeholder
                  0: inplace
                  0: unnamed placeholder
                  "}
    );
}
