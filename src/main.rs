mod services;

use chrono::NaiveDate;
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};

use services::generate_report::generate_reports;
use services::loader::{DataSet, load_file};

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
