use crate::services::loader::DataSet;
use crate::services::reports::report1;
use crate::services::reports::report2;

pub fn generate_reports(data: &Option<DataSet>) {
    match data {
        Some(dataset) => {
            println!("Report 1: Regional Flood Mitigation Efficiency Summary");
            report1::generate_report(dataset);

            println!("Report 2: Top Contractors Performance Ranking");
            report2::generate_report(dataset);

            println!("Report 3: Annual Project Type Cost Overrrun Trends");
            report1::generate_report(dataset);
        }
        None => {
            println!("No dataset loaded. Please load the CSV file first.\n");
        }
    }
}
