use chrono::NaiveDate;
use csv::Reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::Write;

#[derive(Debug)]
struct DataSet {
    total_rows: i32,
    filtered_rows: i32,
    matching_records: Vec<csv::StringRecord>,
    headers: csv::StringRecord,
}

#[derive(Debug)]
struct EfficiencyReport {
    region: String,
    main_island: String,
    total_budget: f64,
    median_savings: f64,
    avg_delay: f64,
    delay_over_30_pct: f64,
    efficiency_score: f64,
}

fn load_file() -> Result<DataSet, Box<dyn Error>> {
    let mut reader = Reader::from_path("./src/data/dpwh_flood_control_projects.csv")?;

    let mut total_rows: i32 = 0;
    let mut filtered_rows: i32 = 0;

    let target_header = "StartDate";
    let headers = reader.headers()?.clone();
    let col_index = headers
        .iter()
        .position(|h| h == target_header)
        .expect("Target Column not found");

    let mut matching_records = Vec::new();

    for result in reader.records() {
        total_rows += 1;
        let record = result?;

        if let Some(date_str) = record.get(col_index) {
            let trimmed = date_str.trim();
            if let Ok(year) = trimmed[0..4].parse::<u32>() {
                if (2021..=2023).contains(&year) {
                    filtered_rows += 1;
                    matching_records.push(record.clone());
                }
            }
        }
    }

    println!(
        "Processing dataset ... ({} rows loaded, {} filtered for {})\n",
        total_rows, filtered_rows, "2021-2023"
    );

    Ok(DataSet {
        total_rows,
        filtered_rows,
        matching_records,
        headers,
    })
}

fn median(mut values: Vec<f64>) -> f64 {
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

fn generate_efficiency_report(data: &DataSet) {
    println!("Regional Flood Mitigation Efficiency Summary");
    println!("Filtered: {}\n", "2021–2023 Projects");

    let headers = &data.headers;
    let get_index = |name: &str| {
        headers
            .iter()
            .position(|h| h == name)
            .unwrap_or_else(|| panic!("Missing column: {}", name))
    };

    let idx_region = get_index("Region");
    let idx_island = get_index("MainIsland");
    let idx_budget = get_index("ApprovedBudgetForContract");
    let idx_contract_cost = get_index("ContractCost");
    let idx_start = get_index("StartDate");
    let idx_end = get_index("ActualCompletionDate");

    let mut groups: HashMap<(String, String), Vec<(f64, f64, f64)>> = HashMap::new();

    for record in &data.matching_records {
        let region = record.get(idx_region).unwrap_or("").trim().to_string();
        let island = record.get(idx_island).unwrap_or("").trim().to_string();

        let budget = record
            .get(idx_budget)
            .unwrap_or("0")
            .replace(",", "")
            .parse::<f64>()
            .unwrap_or(0.0);

        let contract_cost = record
            .get(idx_contract_cost)
            .unwrap_or("0")
            .replace(",", "")
            .parse::<f64>()
            .unwrap_or(0.0);

        let savings = (budget - contract_cost).max(0.0);

        let start_date = record.get(idx_start).unwrap_or("").trim();
        let end_date = record.get(idx_end).unwrap_or("").trim();

        let delay_days = match (
            NaiveDate::parse_from_str(start_date, "%Y-%m-%d").ok(),
            NaiveDate::parse_from_str(end_date, "%Y-%m-%d").ok(),
        ) {
            (Some(s), Some(e)) => (e - s).num_days().max(0) as f64,
            _ => 0.0,
        };

        groups
            .entry((region, island))
            .or_default()
            .push((budget, savings, delay_days));
    }

    let mut reports = Vec::new();

    for ((region, island), values) in groups {
        let total_budget: f64 = values.iter().map(|(b, _, _)| *b).sum();
        let savings_list: Vec<f64> = values.iter().map(|(_, s, _)| *s).collect();
        let delay_list: Vec<f64> = values.iter().map(|(_, _, d)| *d).collect();

        let median_savings = median(savings_list);
        let avg_delay = if delay_list.is_empty() {
            0.0
        } else {
            delay_list.iter().sum::<f64>() / delay_list.len() as f64
        };

        let delay_over_30 = delay_list.iter().filter(|d| **d > 30.0).count() as f64;
        let delay_over_30_pct = if delay_list.is_empty() {
            0.0
        } else {
            (delay_over_30 / delay_list.len() as f64) * 100.0
        };

        let mut efficiency = if avg_delay > 0.0 {
            (median_savings / avg_delay) * 100.0
        } else {
            0.0
        };

        efficiency = efficiency.clamp(0.0, 100.0);

        reports.push(EfficiencyReport {
            region,
            main_island: island,
            total_budget,
            median_savings,
            avg_delay,
            delay_over_30_pct,
            efficiency_score: efficiency,
        });
    }

    // Sort descending by efficiency
    reports.sort_by(|a, b| b.efficiency_score.partial_cmp(&a.efficiency_score).unwrap());

    // Keep top 3
    let top3: Vec<_> = reports.into_iter().take(3).collect();

    println!(
        "{:<20} {:<12} {:>15} {:>15} {:>15} {:>15} {:>15}",
        "Region",
        "Island",
        "Total Budget",
        "Median Savings",
        "Avg Delay",
        "% Delay>30",
        "Efficiency"
    );

    for r in &top3 {
        println!(
            "{:<20} {:<12} {:>15.2} {:>15.2} {:>15.2} {:>15.2} {:>15.2}",
            r.region,
            r.main_island,
            r.total_budget,
            r.median_savings,
            r.avg_delay,
            r.delay_over_30_pct,
            r.efficiency_score
        );
    }

    // Save to CSV
    let mut file = File::create("report1_regional_summary.csv").expect("Failed to create CSV file");
    writeln!(
        file,
        "Region,MainIsland,TotalBudget,MedianSavings,AvgDelay,DelayOver30Pct,EfficiencyScore"
    )
    .unwrap();

    for r in &top3 {
        writeln!(
            file,
            "{},{},{:.2},{:.2},{:.2},{:.2},{:.2}",
            r.region,
            r.main_island,
            r.total_budget,
            r.median_savings,
            r.avg_delay,
            r.delay_over_30_pct,
            r.efficiency_score
        )
        .unwrap();
    }

    println!("\nFull table exported to `report1_regional_summary`\n");
}

fn generate_reports(data: &Option<DataSet>) {
    match data {
        Some(dataset) => {
            println!("Report 1: Regional Flood Mitigation Efficiency Summary\n");
            generate_efficiency_report(dataset);
        }
        None => println!("No dataset loaded. Please load the file first.\n"),
    }
    println!("");
}

fn main() {
    let mut dataset: Option<DataSet> = None;
    loop {
        let mut input = String::new();

        println!("Select Language Implementation");
        println!("[1] Load the file");
        println!("[2] Generate Reports");
        println!("[3] Exit");

        print!("\nEnter Choice: ");
        use std::io::Write;
        io::stdout().flush().unwrap();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let choice: u32 = match input.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Invalid input, please enter a number.\n");
                continue;
            }
        };

        println!("");

        match choice {
            1 => match load_file() {
                Ok(data) => {
                    dataset = Some(data);
                }
                Err(err) => eprintln!("Error: {}\n", err),
            },
            2 => generate_reports(&dataset),
            3 => {
                println!("Exiting Program...");
                break;
            }
            _ => println!("Invalid choice, try again.\n"),
        }
    }
}
