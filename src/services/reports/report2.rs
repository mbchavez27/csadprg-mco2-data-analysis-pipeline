use crate::services::loader::DataSet;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::HashMap;
use tabled::{Table, Tabled};

// -----------------------------
// Data Structures
// -----------------------------
#[derive(Tabled, Debug, Clone, Serialize)]
pub struct ContractorReportRow {
    pub rank: usize,
    pub contractor: String,
    pub total_cost: f64,
    pub num_projects: usize,
    pub avg_delay: f64,
    pub total_savings: f64,
    pub reliability_index: f64,
    pub risk_flag: String,
}

// -----------------------------
// Utility Functions
// -----------------------------
fn parse_f64(v: &str) -> f64 {
    v.replace(",", "").parse::<f64>().unwrap_or(0.0)
}

fn parse_date(v: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(v.trim(), "%Y-%m-%d").ok()
}

fn round2(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

// -----------------------------
// Main Report Generator
// -----------------------------
pub fn generate_report(data: &DataSet) {
    println!("Generating Top Contractors Performance Report...\n");

    let headers = &data.headers;

    let idx_contractor = headers.iter().position(|h| h == "Contractor").unwrap();
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

    let mut contractors: HashMap<String, Vec<(f64, f64, i32)>> = HashMap::new();

    for record in &data.matching_records {
        let contractor = record.get(idx_contractor).unwrap_or("").to_string();
        let budget = parse_f64(record.get(idx_budget).unwrap_or(""));
        let cost = parse_f64(record.get(idx_cost).unwrap_or(""));
        let savings = budget - cost;

        let start = parse_date(record.get(idx_start).unwrap_or(""));
        let actual = parse_date(record.get(idx_actual).unwrap_or(""));

        let delay_days = match (start, actual) {
            (Some(s), Some(a)) => (a - s).num_days() as i32,
            _ => 0,
        };

        contractors
            .entry(contractor)
            .or_default()
            .push((cost, savings, delay_days));
    }

    let mut rows: Vec<ContractorReportRow> = Vec::new();

    for (contractor, projects) in contractors.into_iter() {
        if projects.len() < 5 {
            continue; // filter >=5 projects
        }

        let num_projects = projects.len();
        let avg_delay = projects.iter().map(|p| p.2 as f64).sum::<f64>() / num_projects as f64;
        let total_savings: f64 = projects.iter().map(|p| p.1).sum();
        let total_cost: f64 = projects.iter().map(|p| p.0).sum();

        let mut reliability_index = 0.0;
        if total_cost > 0.0 {
            reliability_index = (1.0 - (avg_delay / 90.0)) * (total_savings / total_cost) * 100.0;
        }
        if reliability_index > 100.0 {
            reliability_index = 100.0;
        }

        let risk_flag = if reliability_index < 50.0 {
            "High Risk".to_string()
        } else {
            "".to_string()
        };

        rows.push(ContractorReportRow {
            rank: 0, // temporary, will assign after sorting
            contractor,
            total_cost: round2(total_cost),
            num_projects,
            avg_delay: round2(avg_delay),
            total_savings: round2(total_savings),
            reliability_index: round2(reliability_index),
            risk_flag,
        });
    }

    // Sort descending by total_cost
    rows.sort_by(|a, b| b.total_cost.partial_cmp(&a.total_cost).unwrap());

    // Assign ranks
    for (i, row) in rows.iter_mut().enumerate() {
        row.rank = i + 1;
    }

    // Keep top 15
    rows.truncate(15);

    // Print table
    let table = Table::new(rows.clone());
    println!("{table}");

    // Export CSV
    let mut wtr = csv::Writer::from_path("report2_contractor_ranking.csv").unwrap();
    for row in rows {
        wtr.serialize(row).unwrap();
    }
    wtr.flush().unwrap();

    println!("Full table exported to report2_contractor_ranking.csv\n");
}
