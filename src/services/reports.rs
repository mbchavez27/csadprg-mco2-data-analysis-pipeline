use crate::services::loader::DataSet;
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

// helper
fn median(mut v: Vec<f64>) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        (v[n / 2 - 1] + v[n / 2]) / 2.0
    }
}

pub fn efficiency_report(data: &DataSet) {
    let mut groups: HashMap<(String, String), Vec<&csv::StringRecord>> = HashMap::new();

    // group records
    for rec in &data.matching_records {
        let region = rec
            .get(data.headers.iter().position(|h| h == "Region").unwrap())
            .unwrap()
            .to_string();
        let island = rec
            .get(data.headers.iter().position(|h| h == "MainIsland").unwrap())
            .unwrap()
            .to_string();
        groups.entry((region, island)).or_default().push(rec);
    }

    let mut reports: Vec<EfficiencyReport> = vec![];
    let mut raw_scores = vec![];

    for ((region, island), rows) in groups {
        let mut total_budget = 0.0;
        let mut savings_list = vec![];
        let mut delays_list = vec![];
        let mut high_delay_count = 0.0;

        let idx_budget = data
            .headers
            .iter()
            .position(|h| h == "ApprovedBudgetForContract")
            .unwrap();
        let idx_savings = data
            .headers
            .iter()
            .position(|h| h == "CostSavings")
            .unwrap();
        let idx_delay = data
            .headers
            .iter()
            .position(|h| h == "CompletionDelayDays")
            .unwrap();

        for r in rows {
            let b = r.get(idx_budget).unwrap().parse::<f64>().unwrap_or(0.0);
            let s = r.get(idx_savings).unwrap().parse::<f64>().unwrap_or(0.0);
            let d = r.get(idx_delay).unwrap().parse::<f64>().unwrap_or(0.0);

            total_budget += b;
            savings_list.push(s);
            delays_list.push(d);
            if d > 30.0 {
                high_delay_count += 1.0;
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
            (high_delay_count / delays_list.len() as f64) * 100.0
        };

        let raw_score = if avg_delay == 0.0 {
            0.0
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
            efficiency_score: raw_score,
        });
    }

    // normalize score 0–100
    let min = raw_scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = raw_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

    for r in &mut reports {
        if (max - min).abs() < 1e-6 {
            r.efficiency_score = 100.0;
        } else {
            r.efficiency_score = ((r.efficiency_score - min) / (max - min)) * 100.0;
        }
    }

    // sort descending
    reports.sort_by(|a, b| b.efficiency_score.partial_cmp(&a.efficiency_score).unwrap());

    // print table
    let table = Table::new(&reports).to_string();
    println!("Regional Flood Mitigation Report Summary\n");
    println!("(Filtered: 2021–2023 Projects)\n");
    println!("{table}");

    // export CSV
    let mut w = Writer::from_path("report1_regional_summary.csv").unwrap();
    w.write_record([
        "Region",
        "MainIsland",
        "TotalBudget",
        "MedianSavings",
        "AvgDelay",
        "HighDelayPct",
        "EfficiencyScore",
    ])
    .unwrap();

    for r in &reports {
        w.write_record(&[
            r.region.to_string(),
            r.main_island.to_string(),
            r.total_budget.to_string(),
            r.median_savings.to_string(),
            r.avg_delay.to_string(),
            r.high_delay_pct.to_string(),
            r.efficiency_score.to_string(),
        ])
        .unwrap();
    }

    w.flush().unwrap();
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
