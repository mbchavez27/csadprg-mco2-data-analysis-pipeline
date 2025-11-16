use crate::services::loader::DataSet;
use crate::services::reports::report1;
use crate::services::reports::report2;
use crate::services::reports::report3;
use std::io::{self, Write};

fn ask_yes_no(prompt: &str) -> bool {
    loop {
        print!("{} (Y/N): ", prompt);
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_uppercase();

        match input.as_str() {
            "Y" => return true,
            "N" => return false,
            _ => println!("Please enter Y or N."),
        }
    }
}

pub fn generate_reports(data: &Option<DataSet>) {
    match data {
        Some(dataset) => loop {
            println!("Report 1: Regional Flood Mitigation Efficiency Summary");
            report1::generate_report(dataset);

            println!("Report 2: Top Contractors Performance Ranking");
            report2::generate_report(dataset);

            println!("Report 3: Annual Project Type Cost Overrun Trends");
            report3::generate_report(dataset);

            if ask_yes_no("Back to Report Selection") {
                break;
            }
        },
        None => {
            println!("No dataset loaded. Please load the CSV file first.\n");
        }
    }
}
