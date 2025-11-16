use crate::services::loader::DataSet;
use chrono::NaiveDate;
use serde::Serialize;
use std::collections::HashMap;
use tabled::{Table, Tabled};

// -----------------------------
// Data Structures
// -----------------------------
#[derive(Tabled, Debug, Clone, Serialize)]
pub struct EfficiencyReportRow {
    pub main_island: String,
    pub region: String,
    pub total_budget: f64,
    pub median_savings: f64,
    pub avg_delay: f64,
    pub delayed_over_30_pct: f64,
    pub efficiency_score: f64,
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

fn median(values: &mut Vec<f64>) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mid = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[mid - 1] + values[mid]) / 2.0
    } else {
        values[mid]
    }
}

fn normalize_to_100(value: f64, max_value: f64) -> f64 {
    if max_value == 0.0 {
        0.0
    } else {
        (value / max_value) * 100.0
    }
}

// -----------------------------
// Main Report Generator
// -----------------------------
pub fn generate_report(data: &DataSet) {
    println!("Generating Efficiency Report...\n");

    let headers = &data.headers;

    let idx_main_island = headers.iter().position(|h| h == "MainIsland").unwrap();
    let idx_region = headers.iter().position(|h| h == "Region").unwrap();
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

    let mut groups: HashMap<(String, String), Vec<(f64, f64, i32)>> = HashMap::new();

    for record in &data.matching_records {
        let main_island = record.get(idx_main_island).unwrap_or("").to_string();
        let region = record.get(idx_region).unwrap_or("").to_string();

        let budget = parse_f64(record.get(idx_budget).unwrap_or(""));
        let cost = parse_f64(record.get(idx_cost).unwrap_or(""));
        let savings = budget - cost;

        let start = parse_date(record.get(idx_start).unwrap_or(""));
        let actual = parse_date(record.get(idx_actual).unwrap_or(""));

        let delay_days = match (start, actual) {
            (Some(s), Some(a)) => (a - s).num_days() as i32,
            _ => 0,
        };

        groups
            .entry((main_island, region))
            .or_default()
            .push((budget, savings, delay_days));
    }

    let mut rows: Vec<EfficiencyReportRow> = Vec::new();
    let mut efficiency_scores: Vec<f64> = Vec::new();

    for ((main_island, region), entries) in groups {
        let total_budget: f64 = entries.iter().map(|e| e.0).sum();
        let mut savings_list: Vec<f64> = entries.iter().map(|e| e.1).collect();
        let med_savings = median(&mut savings_list);

        let avg_delay: f64 = {
            let sum: i64 = entries.iter().map(|e| e.2 as i64).sum();
            sum as f64 / entries.len() as f64
        };

        let delayed_over_30 =
            entries.iter().filter(|e| e.2 > 30).count() as f64 / entries.len() as f64 * 100.0;

        let raw_eff_score = if avg_delay == 0.0 {
            0.0
        } else {
            (med_savings / avg_delay) * 100.0
        };

        efficiency_scores.push(raw_eff_score);

        rows.push(EfficiencyReportRow {
            main_island,
            region,
            total_budget,
            median_savings: med_savings,
            avg_delay,
            delayed_over_30_pct: delayed_over_30,
            efficiency_score: raw_eff_score,
        });
    }

    let max_score = efficiency_scores.iter().cloned().fold(0.0 / 0.0, f64::max);

    for row in &mut rows {
        row.efficiency_score = normalize_to_100(row.efficiency_score, max_score);
    }

    let table = Table::new(rows.clone());
    println!("{table}");

    let mut wtr = csv::Writer::from_path("efficiency_report.csv").unwrap();
    for row in rows {
        wtr.serialize(row).unwrap();
    }
    wtr.flush().unwrap();

    println!("Saved: efficiency_report.csv\n");
}
