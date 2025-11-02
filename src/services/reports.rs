use super::loader::{DataSet, load_file};


pub struct EfficiencyReport {
    region: String,
    main_island: String,
    total_budget: f64,
    median_savings: f64,
    avg_delay: f64,
    delay_over_30_pct: f64,
    efficiency_score: f64,
}

pub fn generate_reports(data: &Option<DataSet>) {
    match data {
        Some(dataset) => {
            println!("Report 1: Regional Flood Mitigation Efficiency Summary\n");
        }
        None => println!("No dataset loaded. Please load the file first.\n"),
    }
    println!("");
}
