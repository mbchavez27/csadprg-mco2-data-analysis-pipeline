# CSADPRG‑MCO2 Data Analysis Pipeline

## Project Overview

The **CSADPRG‑MCO2 Data Analysis Pipeline** is a Rust-based CLI tool designed to analyze flood mitigation project data.  
It processes CSV datasets to generate multiple analytical reports and summary statistics, including:

- Regional Flood Mitigation Efficiency Summary
- Top Contractors Performance Ranking
- Annual Project Type Cost Overrun Trends
- Aggregated summary of project data in JSON

The pipeline outputs results both in the console (tables) and as CSV / JSON files.

## Tech Stack

- **Programming Language:** Rust
- **CSV Processing:** `csv` crate
- **Data & Date Handling:** `chrono` crate
- **Table Display:** `tabled` crate
- **Serialization:** `serde` and `serde_json` crates
