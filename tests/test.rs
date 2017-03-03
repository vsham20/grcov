extern crate walkdir;
extern crate serde_json;

use std::process::Command;
use walkdir::WalkDir;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use serde_json::Value;

fn make(path: &Path) {
    let status = Command::new("make")
                         .current_dir(path)
                         .status()
                         .expect("Failed to build");
    assert!(status.success());
}

fn run(path: &Path) {
    let status = Command::new("./a.out")
                         .current_dir(path)
                         .status()
                         .expect("Failed to run");
    assert!(status.success());
}

fn read_expected(path: &Path) -> Vec<String> {
    let mut f = File::open(path.join("expected.txt")).unwrap();
    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();
    let mut v = Vec::new();
    for line in s.lines() {
        v.push(line.to_string());
    }
    v
}

fn run_grcov(path: &Path) -> Vec<String> {
    let output = Command::new("cargo")
                         .arg("run")
                         .arg(path)
                         .output()
                         .expect("Failed to run grcov");
    let s = String::from_utf8(output.stdout).unwrap();
    let mut v = Vec::new();
    for line in s.lines() {
        v.push(line.to_string());
    }
    v
}

fn make_clean(path: &Path) {
    let status = Command::new("make")
                         .arg("clean")
                         .current_dir(path)
                         .status()
                         .expect("Failed to clean");
    assert!(status.success());
}

fn check_equal(expected_output: Vec<String>, output: Vec<String>) {
    println!("Expected:");
    for line in expected_output.iter() {
        println!("{}", line);
    }
    let mut expected: Vec<Value> = Vec::new();
    for line in expected_output.iter() {
        expected.push(serde_json::from_str(line).unwrap());
    }

    println!("Got:");
    for line in output.iter() {
        println!("{}", line);
    }
    let mut actual: Vec<Value> = Vec::new();
    for line in output.iter() {
        actual.push(serde_json::from_str(line).unwrap());
    }

    for out in actual.iter() {
        let exp = expected.iter().find(|&&ref x| x["sourceFile"] == out["sourceFile"]);
        assert!(exp.is_some(), "Got unexpected {}", out["sourceFile"]);
        let exp_val = exp.unwrap();
        assert_eq!(out["testUrl"], exp_val["testUrl"]);
        assert_eq!(out["covered"], exp_val["covered"]);
        assert_eq!(out["uncovered"], exp_val["uncovered"]);
        assert_eq!(out["methods"].as_object().unwrap().len(), exp_val["methods"].as_object().unwrap().len());
    }

    for exp in expected.iter() {
        let out = actual.iter().find(|&&ref x| x["sourceFile"] == exp["sourceFile"]);
        assert!(out.is_some(), "Missing {}", exp["sourceFile"]);
        assert!(out.is_some());
        let out_val = out.unwrap();
        assert_eq!(exp["testUrl"], out_val["testUrl"]);
        assert_eq!(exp["covered"], out_val["covered"]);
        assert_eq!(exp["uncovered"], out_val["uncovered"]);
        assert_eq!(exp["methods"].as_object().unwrap().len(), out_val["methods"].as_object().unwrap().len());
    }
}

#[test]
fn test_integration() {
    for entry in WalkDir::new("tests").min_depth(1) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            make(path);
            run(path);

            check_equal(read_expected(path), run_grcov(path));

            make_clean(path);
        }
    }
}