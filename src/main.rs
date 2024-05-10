use chrono::{DateTime, Duration, Local};
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

    loop {
        println!("Enter 'start' to start the timer, 'stop' to stop it, 'exit' to exit, or 'show <number>' to display last <number> entries:");
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
                is_running = true;
                println!(
                    "Timer started for: {} at {}",
                    description.trim(),
                    start_time
                );
            }
            "stop" if is_running => {
                let end_time = Local::now();
                let duration = end_time - start_time;
                println!(
                    "Timer stopped for: {} at {}. Total time: {}",
                    description.trim(),
                    end_time,
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
                is_running = false;
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
