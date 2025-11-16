use crate::services::loader::DataSet;
use chrono::NaiveDate;
use csv::Writer;
use std::collections::HashMap;
use tabled::{Table, Tabled};

#[derive(Tabled, Debug, Clone)]
pub struct EfficiencyReport {
    #[tabled(rename = "Region")]
    pub region: String,

    #[tabled(rename = "MainIsland")]
    pub main_island: String,

    #[tabled(rename = "TotalBudget")]
    pub total_budget: f64,

    #[tabled(rename = "MedianSavings")]
    pub median_savings: f64,

    #[tabled(rename = "AvgDelay")]
    pub avg_delay: f64,

    #[tabled(rename = "HighDelayPct")]
    pub high_delay_pct: f64,

    #[tabled(rename = "EfficiencyScore")]
    pub efficiency_score: f64,
}

fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

fn parse_f64_field(record: &csv::StringRecord, idx_opt: Option<usize>) -> f64 {
    if let Some(idx) = idx_opt {
        if let Some(val) = record.get(idx) {
            // remove commas and trim
            let cleaned = val.replace(',', "").trim().to_string();
            return cleaned.parse::<f64>().unwrap_or(0.0);
        }
    }
    0.0
}

fn parse_string_field(record: &csv::StringRecord, idx_opt: Option<usize>) -> String {
    if let Some(idx) = idx_opt {
        if let Some(val) = record.get(idx) {
            return val.trim().to_string();
        }
    }
    "".to_string()
}

fn parse_date_opt(s: &str) -> Option<NaiveDate> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    // Try several common formats. Add more if your data uses different ones.
    let fmts = [
        "%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y", "%Y/%m/%d", "%d-%m-%Y", "%m-%d-%Y",
    ];
    for f in &fmts {
        if let Ok(d) = NaiveDate::parse_from_str(s, f) {
            return Some(d);
        }
    }
    None
}

/// Find header index by exact match (case-sensitive) or by trimmed lower-case fallback.
fn find_idx(headers: &csv::StringRecord, name: &str) -> Option<usize> {
    // exact match first
    if let Some(i) = headers.iter().position(|h| h == name) {
        return Some(i);
    }
    // trimmed, case-insensitive fallback
    let name_l = name.trim().to_lowercase();
    headers
        .iter()
        .position(|h| h.trim().to_lowercase() == name_l)
}

pub fn efficiency_report(data: &DataSet) {
    // helpful print of headers (uncomment if you want to debug)
    // println!("DEBUG: CSV headers = {:?}", data.headers);

    // locate possible columns
    let idx_region = find_idx(&data.headers, "Region");
    let idx_island = find_idx(&data.headers, "MainIsland")
        .or_else(|| find_idx(&data.headers, "Main Island"))
        .or_else(|| find_idx(&data.headers, "MainIsland")); // redundancy OK

    let idx_budget = find_idx(&data.headers, "ApprovedBudgetForContract")
        .or_else(|| find_idx(&data.headers, "ApprovedBudget"))
        .or_else(|| find_idx(&data.headers, "ApprovedBudgetForContractPHP"));

    let idx_contract_cost = find_idx(&data.headers, "ContractCost")
        .or_else(|| find_idx(&data.headers, "Contract Cost"))
        .or_else(|| find_idx(&data.headers, "Contract_Cost"));

    let idx_cost_savings = find_idx(&data.headers, "CostSavings")
        .or_else(|| find_idx(&data.headers, "Cost Savings"))
        .or_else(|| find_idx(&data.headers, "Cost_Savings"));

    let idx_delay_days = find_idx(&data.headers, "CompletionDelayDays")
        .or_else(|| find_idx(&data.headers, "Completion Delay Days"))
        .or_else(|| find_idx(&data.headers, "DelayDays"));

    let idx_actual_completion = find_idx(&data.headers, "ActualCompletionDate")
        .or_else(|| find_idx(&data.headers, "Actual Completion Date"))
        .or_else(|| find_idx(&data.headers, "ActualCompletion"));

    let idx_start_date =
        find_idx(&data.headers, "StartDate").or_else(|| find_idx(&data.headers, "Start Date"));

    // Warn if very important columns are missing (but continue with fallbacks)
    if idx_region.is_none() {
        println!("Warning: 'Region' column not found. Grouping will use empty region strings.");
    }
    if idx_island.is_none() {
        println!(
            "Warning: 'MainIsland' column not found. Grouping will use empty main island strings."
        );
    }
    if idx_budget.is_none() {
        println!(
            "Warning: 'ApprovedBudgetForContract' column not found. TotalBudget will be computed as 0."
        );
    }
    if idx_contract_cost.is_none() {
        println!("Warning: 'ContractCost' column not found. ContractCost will be treated as 0.");
    }
    if idx_cost_savings.is_none() {
        println!(
            "Note: 'CostSavings' column missing — savings will be computed as ApprovedBudgetForContract - ContractCost."
        );
    }

    // Group records by (region, island)
    let mut groups: HashMap<(String, String), Vec<&csv::StringRecord>> = HashMap::new();

    for rec in &data.matching_records {
        let region = parse_string_field(rec, idx_region.clone());
        let island = parse_string_field(rec, idx_island.clone());
        groups.entry((region, island)).or_default().push(rec);
    }

    if groups.is_empty() {
        println!("No records to report on (after filtering).");
        return;
    }

    let mut reports: Vec<EfficiencyReport> = Vec::new();
    let mut raw_scores: Vec<f64> = Vec::new();

    for ((region, island), rows) in groups.into_iter() {
        let mut total_budget = 0.0_f64;
        let mut savings_list: Vec<f64> = Vec::new();
        let mut delays_list: Vec<f64> = Vec::new();
        let mut high_delay_count = 0usize;

        for r in rows {
            let b = parse_f64_field(r, idx_budget);
            let contract_cost = parse_f64_field(r, idx_contract_cost);

            // cost savings: prefer CostSavings column if present, else compute from budget - contract cost
            let s = if idx_cost_savings.is_some() {
                parse_f64_field(r, idx_cost_savings)
            } else {
                // if both budget and contract cost are zero, savings will be 0
                (b - contract_cost)
            };

            // delay: prefer direct column if present, else compute from StartDate -> ActualCompletionDate
            let d = if idx_delay_days.is_some() {
                parse_f64_field(r, idx_delay_days)
            } else {
                // try to compute using dates
                let ac_str = r
                    .get(idx_actual_completion.unwrap_or(usize::MAX))
                    .unwrap_or("")
                    .trim();
                let st_str = r
                    .get(idx_start_date.unwrap_or(usize::MAX))
                    .unwrap_or("")
                    .trim();
                if let (Some(ac), Some(st)) = (parse_date_opt(ac_str), parse_date_opt(st_str)) {
                    let dur = ac.signed_duration_since(st);
                    // convert to days, allow negative -> treat as 0
                    if dur.num_days() >= 0 {
                        dur.num_days() as f64
                    } else {
                        0.0
                    }
                } else {
                    // fallback: if ActualCompletionDate present but not StartDate, can't compute
                    0.0
                }
            };

            total_budget += b;
            savings_list.push(s);
            delays_list.push(d);
            if d > 30.0 {
                high_delay_count += 1;
            }
        }

        let med_savings = median(savings_list);
        let avg_delay = if delays_list.is_empty() {
            0.0
        } else {
            delays_list.iter().sum::<f64>() / delays_list.len() as f64
        };
        let high_delay_pct = if delays_list.is_empty() {
            0.0
        } else {
            (high_delay_count as f64 / delays_list.len() as f64) * 100.0
        };

        // raw score: higher savings and lower delays => higher score.
        // If avg_delay == 0, use med_savings * 100 as an indicator (but clamped later).
        let raw_score = if avg_delay.abs() < std::f64::EPSILON {
            med_savings * 100.0
        } else {
            (med_savings / avg_delay) * 100.0
        };

        raw_scores.push(raw_score);

        reports.push(EfficiencyReport {
            region,
            main_island: island,
            total_budget,
            median_savings: med_savings,
            avg_delay,
            high_delay_pct,
            efficiency_score: raw_score, // will normalize later
        });
    }

    // normalize raw scores to 0..100
    if raw_scores.is_empty() {
        println!("No scores generated (no data).");
        return;
    }
    let min = raw_scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = raw_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    for r in &mut reports {
        if (max - min).abs() < 1e-12 {
            // all equal
            r.efficiency_score = 100.0;
        } else {
            r.efficiency_score = ((r.efficiency_score - min) / (max - min)) * 100.0;
            if r.efficiency_score.is_nan() || r.efficiency_score.is_infinite() {
                r.efficiency_score = 0.0;
            }
        }
    }

    // sort descending by efficiency_score
    reports.sort_by(|a, b| {
        b.efficiency_score
            .partial_cmp(&a.efficiency_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // print table
    println!("Regional Flood Mitigation Report Summary\n");
    println!("(Filtered: 2021–2023 Projects)\n");
    println!("{}", Table::new(&reports).to_string());

    // export CSV
    let mut w = Writer::from_path("report1_regional_summary.csv").unwrap_or_else(|e| {
        panic!(
            "Failed to create CSV file report1_regional_summary.csv: {}",
            e
        );
    });

    w.write_record(&[
        "Region",
        "MainIsland",
        "TotalBudget",
        "MedianSavings",
        "AvgDelay",
        "HighDelayPct",
        "EfficiencyScore",
    ])
    .ok();

    for r in &reports {
        w.write_record(&[
            &r.region,
            &r.main_island,
            &format!("{:.2}", r.total_budget),
            &format!("{:.2}", r.median_savings),
            &format!("{:.2}", r.avg_delay),
            &format!("{:.2}", r.high_delay_pct),
            &format!("{:.2}", r.efficiency_score),
        ])
        .ok();
    }

    w.flush().ok();
    println!("\n(Full table exported to report1_regional_summary.csv)");
}

pub fn generate_reports(data: &Option<DataSet>) {
    match data {
        Some(dataset) => {
            println!("Report 1: Regional Flood Mitigation Efficiency Summary\n");
            efficiency_report(dataset);
        }
        None => println!("No dataset loaded. Please load the file first.\n"),
    }
    println!("");
}
