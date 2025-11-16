use csv::Reader;
use std::error::Error;

#[allow(dead_code)]
pub struct DataSet {
    pub total_rows: i32,
    pub filtered_rows: i32,
    pub matching_records: Vec<csv::StringRecord>,
    pub headers: csv::StringRecord,
}

pub fn load_file() -> Result<DataSet, Box<dyn Error>> {
    //Read the file
    let mut reader = Reader::from_path("data/dpwh_flood_control_projects.csv")?;

    let mut total_rows: i32 = 0;
    let mut filtered_rows: i32 = 0;

    //Get the StartDate
    let headers = reader.headers()?.clone();
    let col_index = headers
        .iter()
        .position(|h| h == "StartDate")
        .expect("Target Column not found");

    let mut matching_records = Vec::new();

    //Add all rows
    for result in reader.records() {
        total_rows += 1;
        let record = result?;

        //Filter rows
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
