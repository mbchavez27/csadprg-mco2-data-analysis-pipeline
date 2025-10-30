use std::io;

fn load_file() {
    println!(
        "Processing dataset ... ({} rows loaded, {} filtered for {})",
        1, 1, 1
    );
}

fn generate_reports() {
    println!("Generating reports...");
}

fn main() {
    loop {
        let mut input = String::new();

        println!("Select Language Implementation");
        println!("[1] Load the file");
        println!("[2] Generate Reports");
        println!("[3] Exit");

        print!("Action: ");
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

        match choice {
            1 => load_file(),
            2 => {}
            3 => {
                println!("Exiting Program...");
                break;
            }
            _ => println!("Invalid choice, try again.\n"),
        }
    }
}
