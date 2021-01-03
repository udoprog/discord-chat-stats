use anyhow::Result;
use plotters::prelude::*;
use std::path::Path;

fn month(x: &SegmentValue<u32>) -> &'static str {
    let x = match *x {
        SegmentValue::Exact(n) | SegmentValue::CenterOf(n) => n,
        _ => return "?",
    };

    match x {
        0 => "Jan",
        1 => "Feb",
        2 => "Mar",
        3 => "Apr",
        4 => "Maj",
        5 => "Jun",
        6 => "Jul",
        7 => "Aug",
        8 => "Sep",
        9 => "Oct",
        10 => "Nov",
        11 => "Dec",
        _ => "?",
    }
}

pub fn month_plot(out: &Path, data: Vec<(u32, u64)>, title: &str) -> Result<()> {
    let root = BitMapBackend::new(out, (640 * 2, 480 * 2)).into_drawing_area();

    let max = data.iter().map(|e| e.1).max().unwrap_or_default();

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(20)
        .caption(title, ("sans-serif", 20.0).into_font())
        .build_cartesian_2d((0u32..11u32).into_segmented(), 0..max)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .x_labels(120)
        .x_label_formatter(&|x| month(x).into())
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(
        Histogram::vertical(&chart)
            .style(RED.mix(0.5).filled())
            .data(data.iter().map(|(m, c)| (*m, *c))),
    )?;

    chart.configure_series_labels().draw()?;
    Ok(())
}

pub fn contributions_per_user(out: &Path, data: Vec<f64>) -> Result<()> {
    let root = BitMapBackend::new(out, (640 * 2, 480 * 2)).into_drawing_area();

    let mut max = 0f64;

    for d in &data {
        if *d > max {
            max = *d;
        }
    }

    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .x_label_area_size(35)
        .y_label_area_size(40)
        .margin(20)
        .caption(
            format!("Contributions per user"),
            ("sans-serif", 20.0).into_font(),
        )
        .build_cartesian_2d(0.9f64..1f64, 0f64..(max + 0.1f64))?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .bold_line_style(&WHITE.mix(0.3))
        .x_labels(32)
        .x_label_formatter(&|y| format!("{}%", (y * 100f64) as u32))
        .y_label_formatter(&|y| format!("{}%", (y * 100f64) as u32))
        .axis_desc_style(("sans-serif", 15))
        .draw()?;

    chart.draw_series(
        AreaSeries::new(
            data.iter()
                .enumerate()
                .map(|(id, c)| ((id as f64) / (data.len() as f64), *c)),
            0.0,
            &RED.mix(0.2),
        )
        .border_style(&RED),
    )?;

    chart.configure_series_labels().draw()?;
    Ok(())
}
