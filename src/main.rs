use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use rusqlite::{params, Connection, Result};
use std::io;

fn main() -> Result<()> {
    let conn = Connection::open("timer.db")?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS timer (
            id INTEGER PRIMARY KEY,
            category TEXT,
            description TEXT,
            detail TEXT,
            start_time TEXT,
            end_time TEXT,
            total_time_seconds INTEGER
        )",
        [],
    )?;

    let mut start_time = Local::now();
    let mut category = String::new();
    let mut description = String::new();
    let mut detail = vec![];
    let mut is_running = false;
    let mut is_paused = false;
    let mut running_duration = Duration::seconds(0);
    let mut last_start_time = Local::now();

    loop {
        println!("Enter 'start' to start the timer, 'stop' to stop it, 'pause' to pause it, 'resume' to resume it, 'manual' to manually input an event, 'exit' to exit, or 'show <number>' to display last <number> entries:");
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        let input = input.trim().to_lowercase();

        if input.starts_with("show") {
            let parts: Vec<&str> = input.split_whitespace().collect();
            if parts.len() == 2 {
                if let Ok(num) = parts[1].parse::<usize>() {
                    show_records(&conn, num)?;
                } else {
                    println!("Please enter a valid number after 'show'.");
                }
            }
            continue;
        }

        match input.as_str() {
            "start" if !is_running => {
                println!("Enter the activity category:");
                io::stdin()
                    .read_line(&mut category)
                    .expect("Failed to read line");
                println!("Enter the activity description:");
                io::stdin()
                    .read_line(&mut description)
                    .expect("Failed to read line");
                start_time = Local::now();
                last_start_time = start_time;
                is_running = true;
                is_paused = false;
                running_duration = Duration::seconds(0);
                detail.clear();
                println!(
                    "Timer started for: {} at {}",
                    description.trim(),
                    start_time
                );
            }
            "pause" if is_running && !is_paused => {
                let pause_time = Local::now();
                let elapsed = pause_time - last_start_time;
                running_duration = running_duration + elapsed;
                is_paused = true;
                println!(
                    "Timer paused for: {} at {}. Total running time: {}",
                    description.trim(),
                    pause_time,
                    format_duration(running_duration)
                );
            }
            "resume" if is_running && is_paused => {
                last_start_time = Local::now();
                is_paused = false;
                println!(
                    "Timer resumed for: {} at {}",
                    description.trim(),
                    last_start_time
                );
            }
            "stop" if is_running => {
                let end_time = Local::now();
                if !is_paused {
                    let elapsed = end_time - last_start_time;
                    running_duration = running_duration + elapsed;
                }
                println!(
                    "Timer stopped for: {} at {}. Total running time: {}",
                    description.trim(),
                    end_time,
                    format_duration(running_duration)
                );
                conn.execute(
                    "INSERT INTO timer (category, description, detail, start_time, end_time, total_time_seconds)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        category.trim(),
                        description.trim(),
                        detail.join("\n"),
                        start_time.to_rfc3339(),
                        end_time.to_rfc3339(),
                        running_duration.num_seconds(),
                    ],
                )?;
                category = String::new();
                description = String::new();
                detail = vec![];
                is_running = false;
                is_paused = false;
            }
            "manual" => {
                println!("Enter the activity category:");
                io::stdin()
                    .read_line(&mut category)
                    .expect("Failed to read line");
                println!("Enter the activity description:");
                io::stdin()
                    .read_line(&mut description)
                    .expect("Failed to read line");

                println!("Enter the start time (e.g., '2024-06-01 14:30'):");
                let mut start_input = String::new();
                io::stdin()
                    .read_line(&mut start_input)
                    .expect("Failed to read line");
                let start_input = start_input.trim();
                let start_time = match parse_time(start_input) {
                    Ok(time) => time,
                    Err(e) => {
                        println!("Error parsing start time: {}", e);
                        category = "".to_string();
                        description = "".to_string();
                        continue;
                    }
                };

                println!("Enter the end time (e.g., '2024-06-01 16:30'):");
                let mut end_input = String::new();
                io::stdin()
                    .read_line(&mut end_input)
                    .expect("Failed to read line");
                let end_input = end_input.trim();
                let end_time = match parse_time(end_input) {
                    Ok(time) => time,
                    Err(e) => {
                        println!("Error parsing end time: {}", e);
                        category = "".to_string();
                        description = "".to_string();
                        continue;
                    }
                };

                let duration = end_time - start_time;
                println!(
                    "Manual entry added for: {}. Total time: {}",
                    description.trim(),
                    format_duration(duration)
                );
                conn.execute(
                    "INSERT INTO timer (category, description, detail, start_time, end_time, total_time_seconds)
                    VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        category.trim(),
                        description.trim(),
                        detail.join("\n"),
                        start_time.to_rfc3339(),
                        end_time.to_rfc3339(),
                        duration.num_seconds(),
                    ],
                )?;
                category = "".to_string();
                description = "".to_string();
            }
            "exit" => break,
            _ => match is_running {
                false => println!("Invalid command."),
                true => {
                    detail.push(input.to_owned());
                }
            },
        }
    }

    Ok(())
}

fn parse_time(input: &str) -> Result<DateTime<Local>, String> {
    let formats = [
        "%Y-%m-%d %H:%M",
        "%Y-%m-%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
        "%Y/%m/%d %H:%M:%S",
    ];

    for format in &formats {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(input, format) {
            return Ok(Local.from_local_datetime(&parsed).unwrap());
        }
    }

    Err("Failed to parse time".to_string())
}

fn format_duration(duration: Duration) -> String {
    format!("{} seconds", duration.num_seconds())
}

fn show_records(conn: &Connection, num_records: usize) -> Result<()> {
    let mut stmt = conn.prepare("SELECT category, description, start_time, end_time, total_time_seconds FROM timer ORDER BY id DESC LIMIT ?1")?;
    let timer_iter = stmt.query_map([num_records], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, i64>(4)?,
        ))
    })?;

    println!("{:=<120}", ""); // Print a line of equals as a header separator
    println!(
        "{:<15} | {:<20} | {:<20} | {:<30} | {:<30}",
        "Category", "Description", "Duration", "Start Time", "End Time",
    );
    println!("{:=<120}", ""); // Repeat header separator after titles

    for timer in timer_iter {
        let (category, description, start_time, end_time, total_time_seconds) = timer?;
        let minutes = total_time_seconds / 60;
        let seconds = total_time_seconds % 60;

        // Safely handle potential parse errors
        let start_dt =
            DateTime::parse_from_str(&format!("{}Z", start_time), "%Y-%m-%dT%H:%M:%S%.fZ");
        let end_dt = DateTime::parse_from_str(&format!("{}Z", end_time), "%Y-%m-%dT%H:%M:%S%.fZ");

        if let (Ok(sd), Ok(ed)) = (start_dt, end_dt) {
            let formatted_start = sd.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S");
            let formatted_end = ed.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S");

            println!(
                "{:<15} | {:<20} | {:>4} min {:02} sec    | {:<30} | {:<30} ",
                category, description, minutes, seconds, formatted_start, formatted_end
            );
        } else {
            println!(
                "{:<15} | {:<20} | {:>3} min {:02} sec    | {:<30} | {:<30} ",
                category, description, minutes, seconds, start_time, end_time
            );
            // println!("Error parsing dates for: {} - {} | Start: {} | End: {}", category, description, start_time, end_time);
        }
    }

    println!("{:=<80}", ""); // Print a closing separator line
    Ok(())
}
