use csv::Reader;
use std::error::Error;
use std::io;

struct DataSet {
    total_rows: i32,
    filtered_rows: i32,
    matching_records: Vec<csv::StringRecord>,
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
    })
}

fn generate_reports(data: &Option<DataSet>) {
    match data {
        Some(dataset) => {
            println!(
                "Generating reports for {} filtered rows out of {} total.\n",
                dataset.filtered_rows, dataset.total_rows
            );

            for (i, record) in dataset.matching_records.iter().take(3).enumerate() {
                println!("Record {}: {:?}", i + 1, record);
            }
        }
        None => println!("No dataset loaded. Please load the file first.\n"),
    }
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
