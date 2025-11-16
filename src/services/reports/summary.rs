use crate::services::loader::DataSet;
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

// -----------------------------
// Data Structure
// -----------------------------
#[derive(Serialize)]
struct Summary {
    total_projects: usize,
    total_contractors: usize,
    total_provinces: usize,
    global_avg_delay: f64,
    total_savings: f64,
}

// -----------------------------
// Generate Summary
// -----------------------------
pub fn generate_summary(data: &DataSet) {
    let headers = &data.headers;

    let idx_contractor = headers.iter().position(|h| h == "Contractor").unwrap();
    let idx_province = headers.iter().position(|h| h == "Province").unwrap();
    let idx_budget = headers
        .iter()
        .position(|h| h == "ApprovedBudgetForContract")
        .unwrap();
    let idx_cost = headers.iter().position(|h| h == "ContractCost").unwrap();
    let idx_start = headers.iter().position(|h| h == "StartDate").unwrap();
    let idx_actual = headers
        .iter()
        .position(|h| h == "ActualCompletionDate")
        .unwrap();

    let mut contractors = HashSet::new();
    let mut provinces = HashSet::new();
    let mut total_delay = 0.0;
    let mut total_savings = 0.0;

    for record in &data.matching_records {
        let contractor = record.get(idx_contractor).unwrap_or("").to_string();
        let province = record.get(idx_province).unwrap_or("").to_string();
        let budget: f64 = record
            .get(idx_budget)
            .unwrap_or("0")
            .replace(",", "")
            .parse()
            .unwrap_or(0.0);
        let cost: f64 = record
            .get(idx_cost)
            .unwrap_or("0")
            .replace(",", "")
            .parse()
            .unwrap_or(0.0);

        contractors.insert(contractor);
        provinces.insert(province);
        total_savings += budget - cost;

        // Compute delay
        let start =
            chrono::NaiveDate::parse_from_str(record.get(idx_start).unwrap_or(""), "%Y-%m-%d").ok();
        let actual =
            chrono::NaiveDate::parse_from_str(record.get(idx_actual).unwrap_or(""), "%Y-%m-%d")
                .ok();
        if let (Some(s), Some(a)) = (start, actual) {
            total_delay += (a - s).num_days() as f64;
        }
    }

    let total_projects = data.matching_records.len();
    let global_avg_delay = if total_projects > 0 {
        total_delay / total_projects as f64
    } else {
        0.0
    };

    let summary = Summary {
        total_projects,
        total_contractors: contractors.len(),
        total_provinces: provinces.len(),
        global_avg_delay,
        total_savings,
    };

    // Convert to pretty JSON
    let json_str = serde_json::to_string_pretty(&summary).unwrap();

    // Print to console
    println!("{json_str}");

    // Save to file
    let mut file = File::create("summary.json").unwrap();
    file.write_all(json_str.as_bytes()).unwrap();

    println!("Summary saved to summary.json");
}
