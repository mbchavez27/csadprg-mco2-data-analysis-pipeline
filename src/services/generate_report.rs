use crate::services::loader::DataSet;
use crate::services::reports::report1;

pub fn generate_reports(data: &Option<DataSet>) {
    match data {
        Some(dataset) => {
            println!("Report 1: Regional Flood Mitigation Efficiency Summary");
            report1::generate_report(dataset);

            println!("Report 2: <Your second report name here>");
            report1::generate_report(dataset);

            println!("Report 3: <Your third report name here>");
            report1::generate_report(dataset);
        }
        None => {
            println!("No dataset loaded. Please load the CSV file first.\n");
        }
    }
}
