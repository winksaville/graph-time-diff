use chrono::NaiveDateTime;
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

    // Ensure dates are sorted.
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

    // Define the x-axis range in seconds (converted to f64 for plotting).
    let x_min = 0.0;
    let x_max = dates.last().unwrap().timestamp() as f64 - first_date.timestamp() as f64;

    // Determine if the data spans less than a day.
    let span_less_than_day = x_max < 86400.0;
    // Set major tick interval: hourly if less than a day, daily if more.
    let major_tick_interval = if span_less_than_day { 3600.0 } else { 86400.0 };

    let first_ts = first_date.timestamp();
    // Compute the first major tick (aligned to the next hour or day boundary).
    let first_major_tick = if span_less_than_day {
        (((first_ts as f64 + 3600.0 - 1.0) / 3600.0).floor() * 3600.0) - first_ts as f64
    } else {
        (((first_ts as f64 + 86400.0 - 1.0) / 86400.0).floor() * 86400.0) - first_ts as f64
    };

    // Compute major tick positions (in seconds offset) for potential manual drawing.
    let mut major_ticks = Vec::new();
    let mut tick = first_major_tick;
    while tick <= x_max {
        major_ticks.push(tick);
        tick += major_tick_interval;
    }

    // Create drawing area with 2048×1024 dimensions.
    let root = BitMapBackend::new("output.png", (2048, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    // Set up the x-axis label formatter.
    let major_tick_format = if span_less_than_day {
        Box::new(move |x: &f64| {
            let dt = NaiveDateTime::from_timestamp(first_ts + *x as i64, 0);
            dt.format("%H:%M:%S").to_string()
        }) as Box<dyn Fn(&f64) -> String>
    } else {
        Box::new(move |x: &f64| {
            let dt = NaiveDateTime::from_timestamp(first_ts + *x as i64, 0);
            format!("{} - {}", dt.format("%b %d"), dt.format("%H:%M:%S"))
        }) as Box<dyn Fn(&f64) -> String>
    };

    // Determine y-axis range from the computed minute differences.
    let y_min = data.iter().map(|&(_, diff)| diff).fold(f64::INFINITY, f64::min) - 0.1;
    let y_max = data.iter().map(|&(_, diff)| diff).fold(f64::NEG_INFINITY, f64::max) + 0.1;

    // Build the chart using f64 for the x-axis.
    let mut chart = ChartBuilder::on(&root)
        .caption("Date Differences in Minutes", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(50)
        .y_label_area_size(50)
        .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

    // Configure the mesh with our custom x-axis label formatter.
    // Using default tick generation.
    chart.configure_mesh()
        .x_desc("Time (since first date)")
        .y_desc("Difference (minutes)")
        .x_label_formatter(&*major_tick_format)
        .draw()?;

    // Draw the series as a line plot.
    chart.draw_series(LineSeries::new(
        data.iter().map(|&(x, y)| (x as f64, y)),
        &RED,
    ))?
    .label("Diff (min)")
    .legend(|(x, y)| Path::new(vec![(x, y), (x + 20, y)], &RED));

    // Draw the legend.
    chart.configure_series_labels()
        .border_style(&BLACK)
        .draw()?;

    println!("Plot saved to output.png");

    Ok(())
}
