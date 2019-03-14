use pdfpdf::{Alignment::*, Color, Matrix, Pdf};

pub struct Plot {
    pdf: Pdf,
    width: f64,
    height: f64,
    tick_length: f64,
    x_tick_interval: Option<f64>,
    y_tick_interval: Option<f64>,
    xlim: Option<(f64, f64)>,
    ylim: Option<(f64, f64)>,
    xlabel: Option<String>,
    ylabel: Option<String>,
}

#[derive(Debug, Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}

trait ToU64 {
    fn to_u64(self) -> u64;
}

impl ToU64 for f64 {
    fn to_u64(self) -> u64 {
        assert!(
            self >= u64::min_value() as f64,
            "{} < u64::min_value(), {}",
            self,
            u64::min_value()
        );
        assert!(
            self <= u64::max_value() as f64
            "{} > u64::max_value(), {}",
            self,
            u64::max_value()
        );
        self as u64
    }
}

trait FloatMax {
    fn float_max(self) -> f64;
}

impl<T> FloatMax for T
where
    T: Iterator<Item = f64>,
{
    fn float_max(mut self) -> f64 {
        let mut max = self.next().unwrap();
        for item in self {
            if item > max {
                max = item;
            }
        }
        max
    }
}

fn compute_tick_interval(range: f64) -> f64 {
    let range = range.abs();
    let order_of_magnitude = (10.0f64).powi(range.log10() as i32);
    let possible_tick_intervals = [
        order_of_magnitude / 2.0,
        order_of_magnitude,
        order_of_magnitude * 2.0,
    ];
    let num_ticks = [
        (range / possible_tick_intervals[0]).round() as i64,
        (range / possible_tick_intervals[1]).round() as i64,
        (range / possible_tick_intervals[2]).round() as i64,
    ];
    // Try to get as close to 5 ticks as possible
    let chosen_index = num_ticks
        .iter()
        .enumerate()
        .min_by_key(|(_, num)| (**num - 5).abs())
        .unwrap()
        .0;
    possible_tick_intervals[chosen_index]
}

impl Plot {
    pub fn new() -> Self {
        Self {
            pdf: Pdf::new_uncompressed(),
            width: 500.0,
            height: 500.0,
            tick_length: 6.0,
            x_tick_interval: None,
            y_tick_interval: None,
            xlim: None,
            ylim: None,
            xlabel: None,
            ylabel: None,
        }
    }

    pub fn ylim(&mut self, min: f64, max: f64) -> &mut Self {
        self.ylim = Some((min, max));
        self
    }

    pub fn xlim(&mut self, min: f64, max: f64) -> &mut Self {
        self.xlim = Some((min, max));
        self
    }

    pub fn xlabel(&mut self, text: &str) -> &mut Self {
        self.xlabel = Some(text.to_string());
        self
    }

    pub fn ylabel(&mut self, text: &str) -> &mut Self {
        self.ylabel = Some(text.to_string());
        self
    }

    pub fn tick_length(&mut self, length: f64) -> &mut Self {
        self.tick_length = length;
        self
    }

    pub fn plot(&mut self, x_values: &[f64], y_values: &[f64]) -> &mut Self {
        // Pick the axes limits
        let (min, max) = {
            use std::f64;
            let mut max = Point {
                x: f64::NEG_INFINITY,
                y: f64::NEG_INFINITY,
            };
            let mut min = Point {
                x: f64::INFINITY,
                y: f64::INFINITY,
            };
            for (&x, &y) in x_values.iter().zip(y_values.iter()) {
                max.x = max.x.max(x);
                max.y = max.y.max(y);
                min.x = min.x.min(x);
                min.y = min.y.min(y);
            }
            (min, max)
        };

        assert!(max.x.is_finite());
        assert!(max.y.is_finite());
        assert!(min.x.is_finite());
        assert!(min.y.is_finite());

        // Compute the tick labels for x and y axes so that we can position the plotting area
        let x_tick_interval = self
            .x_tick_interval
            .unwrap_or(compute_tick_interval(max.x - min.x));

        let y_tick_interval = self
            .y_tick_interval
            .unwrap_or(compute_tick_interval(max.y - min.y));

        let xlim = self.xlim.unwrap_or_else(|| {
            let min_in_ticks = (min.x / x_tick_interval).floor();
            let xmin = min_in_ticks * x_tick_interval;
            let max_in_ticks = (max.x / x_tick_interval).ceil();
            let xmax = max_in_ticks * x_tick_interval;
            (xmin, xmax)
        });

        let ylim = self.ylim.unwrap_or_else(|| {
            let min_in_ticks = (min.y / y_tick_interval).floor();
            let ymin = min_in_ticks * y_tick_interval;
            let max_in_ticks = (max.y / y_tick_interval).ceil();
            let ymax = max_in_ticks * y_tick_interval;
            (ymin, ymax)
        });

        let x_num_ticks = ((xlim.1 - xlim.0).abs() / x_tick_interval).to_u64() + 1;
        let y_num_ticks = ((ylim.1 - ylim.0).abs() / y_tick_interval).to_u64() + 1;

        let x_tick_interval = x_tick_interval * (xlim.1 - xlim.0).signum();
        let y_tick_interval = y_tick_interval * (ylim.1 - ylim.0).signum();

        // Y Border size is height of the font, max width of a label, and the tick length
        let yaxis_margin = 12. * 2.
            + (0..y_num_ticks)
                .map(|i| i as f64 * y_tick_interval + ylim.0)
                .map(|v| self.pdf.width_of(&format!("{}", v)))
                .float_max()
            + self.tick_length;

        // X border size is 1.5 * height of the axis label label, height of the tick labels, and the tick length
        let xaxis_margin = (12. * 1.5) + 12. + self.tick_length;

        let width = self.width;
        let height = self.height;

        // Function to convert from plot pixels to canvas pixels
        let to_canvas_x = |x| {
            let plot_width = width - yaxis_margin - 0.075 * width;
            let x_scale = plot_width / (xlim.1 - xlim.0);
            ((x - xlim.0) * x_scale) + yaxis_margin
        };

        let to_canvas_y = |y| {
            let plot_height = height - xaxis_margin - 0.075 * height;
            let y_scale = plot_height / (ylim.1 - ylim.0);
            ((y - ylim.0) * y_scale) + xaxis_margin
        };

        // Draw the plot's border at the margins
        self.pdf
            .add_page(self.width, self.height)
            .set_color(Color::gray(0))
            .set_line_width(0.75)
            .move_to(to_canvas_x(xlim.0), to_canvas_y(ylim.1))
            .line_to(to_canvas_x(xlim.0), to_canvas_y(ylim.0))
            .line_to(to_canvas_x(xlim.1), to_canvas_y(ylim.0))
            .end_line();

        // Draw the x tick marks
        for i in 0..x_num_ticks {
            let x = i as f64 * x_tick_interval + xlim.0;
            self.pdf
                .move_to(to_canvas_x(x), to_canvas_y(ylim.0))
                .line_to(to_canvas_x(x), to_canvas_y(ylim.0) - self.tick_length)
                .end_line();
            self.pdf.draw_text(
                to_canvas_x(x),
                to_canvas_y(ylim.0) - self.tick_length,
                TopCenter,
                &format!("{}", x),
            );
        }

        // Draw the y tick marks
        for i in 0..y_num_ticks {
            let y = i as f64 * y_tick_interval + ylim.0;
            self.pdf
                .move_to(to_canvas_x(xlim.0), to_canvas_y(y))
                .line_to(to_canvas_x(xlim.0) - self.tick_length, to_canvas_y(y))
                .end_line();
            self.pdf.draw_text(
                to_canvas_x(xlim.0) - self.tick_length - 2.0,
                to_canvas_y(y),
                CenterRight,
                &format!("{}", y),
            );
        }

        let scaled = x_values
            .iter()
            .zip(y_values.iter())
            //.filter(|(&x, &y)| x >= xlim.0 && x <= xlim.1 && y >= ylim.0 && y <= ylim.1)
            .map(|(&x, &y)| (to_canvas_x(x), to_canvas_y(y)));

        // Draw the data series
        self.pdf
            .set_line_width(1.5)
            .set_color(Color::rgb(31, 119, 180))
            .draw_line(scaled)
            .set_color(Color::gray(0));

        // Draw the x label
        if let Some(ref xlabel) = self.xlabel {
            self.pdf.draw_text(
                to_canvas_x(xlim.0 + (xlim.1 - xlim.0) / 2.0),
                2,
                BottomCenter,
                xlabel,
            );
        }
        if let Some(ref ylabel) = self.ylabel {
            // Draw the y label
            self.pdf.transform(Matrix::rotate_deg(90)).draw_text(
                to_canvas_y(ylim.0 + (ylim.1 - ylim.0) / 2.0),
                0,
                TopCenter,
                ylabel,
            );
        }

        self
    }

    pub fn write_to(&mut self, filename: &str) -> std::io::Result<()> {
        self.pdf.write_to(filename)
    }
}
