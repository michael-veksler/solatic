use indoc::indoc;
use solatic::{dprint, dprint_once, dprintln, dprintln_once};
use std::io;
use std::process::Command;

#[test]
fn dprint_dprintln_output() {
    for i in 0..=4 {
        let mut maybe_stderr: Option<&mut dyn io::Write> = if i % 2 == 0 { Some(&mut io::stderr()) } else { None };
        dprintln!(maybe_stderr, "{index}: named placeholder", index = i);
        dprintln!(maybe_stderr, "{i}: inplace");
        dprint!(maybe_stderr, "{}: ", i);
        dprintln!(maybe_stderr, "unnamed placeholder");
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
        let mut maybe_stderr: Option<&mut dyn io::Write> = if i % 2 == 0 { Some(&mut io::stderr()) } else { None };
        dprintln_once!(maybe_stderr, "{index}: named placeholder", index = i);
        dprintln_once!(maybe_stderr, "{i}: inplace");
        dprint_once!(maybe_stderr, "{}: ", i);
        dprintln_once!(maybe_stderr, "unnamed placeholder");
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
