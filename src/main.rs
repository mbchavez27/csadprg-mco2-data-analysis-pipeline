use csv::Reader;
use std::error::Error;
use std::io;

fn load_file() -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

fn generate_reports() {
    println!("Generating reports...\n");
}

fn main() {
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
            1 => {
                if let Err(err) = load_file() {
                    eprintln!("Error: {}", err);
                }
            }
            2 => generate_reports(),
            3 => {
                println!("Exiting Program...");
                break;
            }
            _ => println!("Invalid choice, try again.\n"),
        }
    }
}
