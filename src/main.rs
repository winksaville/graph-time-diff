use chrono::{NaiveDateTime, Duration};
use plotters::prelude::*;
use std::error::Error;
use std::fs;

fn main() -> Result<(), Box<dyn Error>> {
    // Read the file containing dates.
    let content = fs::read_to_string("dates.txt")?;
    let mut dates: Vec<NaiveDateTime> = Vec::new();

    // Parse each non-empty line, assuming a fixed year (e.g., 2025).
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Prepend a year so we can parse the month/day/time.
        let full_date = format!("2025 {}", line);
        let dt = NaiveDateTime::parse_from_str(&full_date, "%Y %b %d %H:%M:%S")?;
        dates.push(dt);
    }

    if dates.len() < 2 {
        return Err("Need at least two dates to compute differences".into());
    }

    // Ensure dates are sorted (if not already).
    dates.sort();

    // Compute differences between adjacent dates in minutes (as f64).
    // We plot the point corresponding to the later date of each pair.
    let first_date = dates[0];
    let data: Vec<(i64, f64)> = dates.windows(2)
        .map(|window| {
            let dt_current = window[1];
            let diff_seconds = dt_current.signed_duration_since(window[0]).num_seconds();
            let diff_minutes = diff_seconds as f64 / 60.0;
            // x-value: seconds elapsed since the first date.
            let x_val = dt_current.timestamp() - first_date.timestamp();
            (x_val, diff_minutes)
        })
        .collect();

    // Define the x-axis range (in seconds from the first date).
    let x_min = 0;
    let x_max = dates.last().unwrap().timestamp() - first_date.timestamp();
    // y-axis range based on the computed differences in minutes.
    let y_min = data.iter().map(|&(_, diff)| diff).fold(f64::INFINITY, f64::min) - 0.1;
    let y_max = data.iter().map(|&(_, diff)| diff).fold(f64::NEG_INFINITY, f64::max) + 0.1;

    // Create a drawing area (output image).
    let root = BitMapBackend::new("output.png", (640, 480)).into_drawing_area();
    root.fill(&WHITE)?;

    let first_ts = first_date.timestamp();
    let mut chart = ChartBuilder::on(&root)
        .caption("Date Differences in Minutes", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    chart.configure_mesh()
        .x_desc("Time (since first date)")
        .y_desc("Difference (minutes)")
        .x_label_formatter(&move |x| {
            // Convert x (seconds offset) back to a time.
            let dt = NaiveDateTime::from_timestamp(first_ts + *x, 0);
            dt.format("%H:%M:%S").to_string()
        })
        .draw()?;

    // Draw the series as a line plot.
    chart.draw_series(LineSeries::new(
        data.iter().map(|&(x, y)| (x, y)),
        &RED,
    ))?
    .label("Diff (min)")
    .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &RED));

    // Optional: add a legend.
    chart.configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    println!("Plot saved to output.png");

    Ok(())
}
