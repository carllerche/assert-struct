#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct DataSet {
    values: Vec<i32>,
    tags: Vec<String>,
}

#[derive(Debug)]
struct Analysis {
    name: String,
    data: DataSet,
}

#[derive(Debug)]
struct Report {
    title: String,
    analysis: Analysis,
}

pub fn test_case() {
    let report = Report {
        title: "Q4 Report".to_string(),
        analysis: Analysis {
            name: "Revenue".to_string(),
            data: DataSet {
                values: vec![10, 20, 30],
                tags: vec!["a".to_string(), "b".to_string()],
            },
        },
    };

    assert_struct!(report, Report {
        title: "Q4 Report",
        analysis: Analysis {
            name: "Revenue",
            data: DataSet {
                values: [10, 25, 30],  // Line 38 - should report this line (20 != 25)
                tags: ["a", "b"],
            },
        },
    });
}