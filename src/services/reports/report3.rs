use crate::services::loader::DataSet;
use serde::Serialize;
use std::collections::HashMap;
use tabled::{Table, Tabled};

// -----------------------------
// Data Structures
// -----------------------------
#[derive(Tabled, Debug, Clone, Serialize)]
pub struct ProjectTypeReportRow {
    pub funding_year: String,
    pub type_of_work: String,
    pub total_projects: usize,
    pub avg_savings: f64,
    pub overrun_rate: f64,
    pub yoy_change: f64,
}

// -----------------------------
// Utility Functions
// -----------------------------
fn parse_f64(v: &str) -> f64 {
    v.replace(",", "").parse::<f64>().unwrap_or(0.0)
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// -----------------------------
// Main Report Generator
// -----------------------------
pub fn generate_report(data: &DataSet) {
    println!("Generating Annual Project Type Cost Overrun Trends...\n");

    let headers = &data.headers;

    let idx_year = headers.iter().position(|h| h == "FundingYear").unwrap();
    let idx_type = headers.iter().position(|h| h == "TypeOfWork").unwrap();
    let idx_budget = headers
        .iter()
        .position(|h| h == "ApprovedBudgetForContract")
        .unwrap();
    let idx_cost = headers.iter().position(|h| h == "ContractCost").unwrap();

    // Group by (FundingYear, TypeOfWork)
    let mut groups: HashMap<(String, String), Vec<f64>> = HashMap::new();

    for record in &data.matching_records {
        let year = record.get(idx_year).unwrap_or("").to_string();
        let work_type = record.get(idx_type).unwrap_or("").to_string();
        let budget = parse_f64(record.get(idx_budget).unwrap_or(""));
        let cost = parse_f64(record.get(idx_cost).unwrap_or(""));
        let savings = budget - cost; // negative if overrun

        groups.entry((year, work_type)).or_default().push(savings);
    }

    // Collect unique years in ascending order
    let mut years: Vec<String> = groups.keys().map(|(y, _)| y.clone()).collect();
    years.sort();
    years.dedup();

    // Baselines for YoY (2021)
    let mut baselines: HashMap<String, f64> = HashMap::new();
    let mut rows: Vec<ProjectTypeReportRow> = Vec::new();

    for year in &years {
        let year_groups: Vec<_> = groups.iter().filter(|((y, _), _)| y == year).collect();

        for ((_, work_type), savings_list) in year_groups {
            let total_projects = savings_list.len();
            let avg_savings = if total_projects == 0 {
                0.0
            } else {
                savings_list.iter().sum::<f64>() / total_projects as f64
            };
            let overrun_count = savings_list.iter().filter(|s| **s < 0.0).count();
            let overrun_rate = if total_projects == 0 {
                0.0
            } else {
                overrun_count as f64 / total_projects as f64 * 100.0
            };

            // YoY change
            let yoy_change = if year == "2021" {
                baselines.insert(work_type.clone(), avg_savings);
                0.0
            } else {
                let baseline = baselines.get(work_type).cloned().unwrap_or(avg_savings);
                if baseline.abs() < f64::EPSILON {
                    0.0
                } else {
                    (avg_savings - baseline) / baseline * 100.0
                }
            };

            rows.push(ProjectTypeReportRow {
                funding_year: year.clone(),
                type_of_work: work_type.clone(),
                total_projects,
                avg_savings: round2(avg_savings),
                overrun_rate: round2(overrun_rate),
                yoy_change: round2(yoy_change),
            });
        }
    }

    // Sort by FundingYear ascending, AvgSavings descending
    rows.sort_by(|a, b| {
        a.funding_year
            .cmp(&b.funding_year)
            .then(b.avg_savings.partial_cmp(&a.avg_savings).unwrap())
    });

    // Assign table
    let table = Table::new(rows.clone());
    println!("{table}");

    // Export CSV
    let mut wtr = csv::Writer::from_path("report3_project_type_trends.csv").unwrap();
    for row in rows {
        wtr.serialize(row).unwrap();
    }
    wtr.flush().unwrap();

    println!("Full table exported to report3_project_type_trends.csv\n");
}
