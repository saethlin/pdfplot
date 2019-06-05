mod util;
pub use util::loadtxt;
use util::{FloatMax, ToU64};

mod colormaps;

use pdfpdf::{Alignment::*, Color, Matrix, Pdf, Point, Size};

pub struct Plot {
    pdf: Pdf,
    width: f64,
    height: f64,
    font_size: f64,
    tick_length: f64,
    x_tick_interval: Option<f64>,
    y_tick_interval: Option<f64>,
    xlim: Option<(f64, f64)>,
    ylim: Option<(f64, f64)>,
    xlabel: Option<String>,
    ylabel: Option<String>,
    marker: Option<Marker>,
    linestyle: Option<LineStyle>,
}

#[derive(Clone, Copy, Debug)]
pub enum Marker {
    Dot,
}

#[derive(Clone, Copy, Debug)]
pub enum LineStyle {
    Solid,
}

fn compute_tick_interval(range: f64) -> f64 {
    let range = range.abs();
    let order_of_magnitude = (10.0f64).powi(range.log10().round() as i32);
    let possible_tick_intervals = [
        order_of_magnitude / 10.0,
        order_of_magnitude / 5.0,
        order_of_magnitude / 2.0,
        order_of_magnitude,
        order_of_magnitude * 2.0,
    ];
    let num_ticks = [
        (range / possible_tick_intervals[0]).round() as i64,
        (range / possible_tick_intervals[1]).round() as i64,
        (range / possible_tick_intervals[2]).round() as i64,
        (range / possible_tick_intervals[3]).round() as i64,
        (range / possible_tick_intervals[4]).round() as i64,
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

struct Axis {
    limits: (f64, f64),
    tick_interval: f64,
    num_ticks: u64,
    tick_labels: Vec<String>,
    margin: f64,
}

impl Axis {
    fn tick_labels(&mut self) {
        let tick_precision = self.tick_interval.abs().log10();
        let tick_max = self.limits.0.abs().max(self.limits.1.abs()).log10();

        self.tick_labels = (0..self.num_ticks)
            .map(|i| i as f64 * self.tick_interval + self.limits.0)
            .map(|v| {
                if v == 0.0 {
                    format!("{}", v)
                } else if tick_precision < 0.0 {
                    // If we have small ticks, format so that the last sig fig is visible
                    format!("{:.*}", tick_precision.abs().ceil() as usize, v)
                } else if tick_max < 4. {
                    // For numbers close to +/- 1, use default formatting
                    format!("{:.2}", v)
                } else {
                    format!(
                        "{:.*e}",
                        ((tick_max - tick_precision).abs().ceil() - 1.).max(1.) as usize,
                        v
                    )
                }
            })
            .collect();
    }
}

impl Plot {
    pub fn new() -> Self {
        let mut pdf = Pdf::new();
        pdf.font(pdfpdf::Font::Helvetica, 12.0).precision(4);
        Self {
            pdf,
            font_size: 20.0,
            width: 810.0,
            height: 630.0,
            tick_length: 10.0,
            x_tick_interval: None,
            y_tick_interval: None,
            xlim: None,
            ylim: None,
            xlabel: None,
            ylabel: None,
            marker: None,
            linestyle: Some(LineStyle::Solid),
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

    pub fn x_tick_interval(&mut self, interval: f64) -> &mut Self {
        self.x_tick_interval = Some(interval);
        self
    }

    pub fn y_tick_interval(&mut self, interval: f64) -> &mut Self {
        self.y_tick_interval = Some(interval);
        self
    }

    pub fn marker(&mut self, marker: Option<Marker>) -> &mut Self {
        self.marker = marker;
        self
    }

    pub fn linestyle(&mut self, style: Option<LineStyle>) -> &mut Self {
        self.linestyle = style;
        self
    }

    fn digest_tick_settings(&self, x_values: &[f64], y_values: &[f64]) -> (Axis, Axis) {
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

        // Must either provide data or configure
        assert!((min.x.is_finite() && max.x.is_finite()) || self.xlim.is_some());
        assert!((min.y.is_finite() && max.y.is_finite()) || self.ylim.is_some());

        // Compute the tick interval from maxes first so we can choose limits that are a multiple
        // of the tick interval
        let x_tick_interval = self
            .x_tick_interval
            .unwrap_or_else(|| compute_tick_interval(max.x - min.x));

        let y_tick_interval = self
            .y_tick_interval
            .unwrap_or_else(|| compute_tick_interval(max.y - min.y));

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

        // Compute the tick interval again but this time based on the now-known axes limits
        // This fixes our selection of tick interval in situations where we were told odd axes
        // limits
        let x_tick_interval = self
            .x_tick_interval
            .unwrap_or_else(|| compute_tick_interval(xlim.1 - xlim.0));

        let y_tick_interval = self
            .y_tick_interval
            .unwrap_or_else(|| compute_tick_interval(ylim.1 - ylim.0));

        let x_num_ticks = ((xlim.1 - xlim.0).abs() / x_tick_interval).to_u64() + 1;
        let y_num_ticks = ((ylim.1 - ylim.0).abs() / y_tick_interval).to_u64() + 1;

        // Quantize the tick interval so that it fits nicely
        let x_tick_interval = x_tick_interval * (xlim.1 - xlim.0).signum();
        let y_tick_interval = y_tick_interval * (ylim.1 - ylim.0).signum();

        let mut xaxis = Axis {
            limits: xlim,
            num_ticks: x_num_ticks,
            tick_interval: x_tick_interval,
            margin: 0.0,
            tick_labels: Vec::new(),
        };
        xaxis.tick_labels();

        // X border size is 1.5 * height of the axis label label, height of the tick labels, and the tick length
        xaxis.margin = (self.font_size * 1.5) + self.font_size + self.tick_length + self.font_size;

        let mut yaxis = Axis {
            limits: ylim,
            num_ticks: y_num_ticks,
            tick_interval: y_tick_interval,
            margin: 0.0,
            tick_labels: Vec::new(),
        };
        yaxis.tick_labels();

        // Y Border size is height of the font, max width of a label, and the tick length
        yaxis.margin = self.font_size * 2.
            + yaxis
                .tick_labels
                .iter()
                .map(|label| self.pdf.width_of(&label))
                .float_max()
            + self.tick_length
            + self.font_size;

        (xaxis, yaxis)
    }

    fn draw_axes(
        &mut self,
        xaxis: &Axis,
        yaxis: &Axis,
        to_canvas_x: impl Fn(f64) -> f64,
        to_canvas_y: impl Fn(f64) -> f64,
    ) {
        // Draw the plot's border at the margins
        self.pdf
            .add_page(Size {
                width: self.width,
                height: self.height,
            })
            .set_color(Color::gray(0))
            .set_line_width(1.0)
            .draw_rectangle(
                Point {
                    x: to_canvas_x(xaxis.limits.0),
                    y: to_canvas_y(yaxis.limits.0),
                },
                Size {
                    width: to_canvas_x(xaxis.limits.1) - to_canvas_x(xaxis.limits.0),
                    height: to_canvas_y(yaxis.limits.1) - to_canvas_y(yaxis.limits.0),
                },
            );

        // Draw the x tick marks
        for (i, label) in (0..xaxis.num_ticks).zip(&xaxis.tick_labels) {
            let x = i as f64 * xaxis.tick_interval + xaxis.limits.0;
            self.pdf
                .move_to(Point {
                    x: to_canvas_x(x),
                    y: to_canvas_y(yaxis.limits.0),
                })
                .line_to(Point {
                    x: to_canvas_x(x),
                    y: to_canvas_y(yaxis.limits.0) - self.tick_length,
                })
                .end_line();
            self.pdf.draw_text(
                Point {
                    x: to_canvas_x(x),
                    y: to_canvas_y(yaxis.limits.0) - self.tick_length,
                },
                TopCenter,
                label,
            );
        }

        // Draw the y tick marks
        for (i, label) in (0..yaxis.num_ticks).zip(&yaxis.tick_labels) {
            let y = i as f64 * yaxis.tick_interval + yaxis.limits.0;
            self.pdf
                .move_to(Point {
                    x: to_canvas_x(xaxis.limits.0),
                    y: to_canvas_y(y),
                })
                .line_to(Point {
                    x: to_canvas_x(xaxis.limits.0) - self.tick_length,
                    y: to_canvas_y(y),
                })
                .end_line();
            self.pdf.draw_text(
                Point {
                    x: to_canvas_x(xaxis.limits.0) - self.tick_length - 2.0,
                    y: to_canvas_y(y),
                },
                CenterRight,
                label,
            );
        }

        // Draw the x label
        if let Some(ref xlabel) = self.xlabel {
            self.pdf.draw_text(
                Point {
                    x: to_canvas_x(xaxis.limits.0 + (xaxis.limits.1 - xaxis.limits.0) / 2.0),
                    y: 4.0 + self.font_size / 2.0,
                },
                BottomCenter,
                xlabel,
            );
        }

        // Draw the y label
        if let Some(ref ylabel) = self.ylabel {
            self.pdf.transform(Matrix::rotate_deg(90)).draw_text(
                Point {
                    x: to_canvas_y(yaxis.limits.0 + (yaxis.limits.1 - yaxis.limits.0) / 2.0),
                    y: -6.0,
                },
                TopCenter,
                ylabel,
            );
            self.pdf.transform(Matrix::rotate_deg(-90));
        }
    }

    pub fn plot(&mut self, x_values: &[f64], y_values: &[f64]) -> &mut Self {
        let (xaxis, yaxis) = self.digest_tick_settings(x_values, y_values);

        let width = self.width;
        let height = self.height;

        let plot_width =
            width - yaxis.margin - self.pdf.width_of(xaxis.tick_labels.last().unwrap());
        let plot_height = height - xaxis.margin - self.font_size;

        // Function to convert from plot pixels to canvas pixels
        let to_canvas_x = |x| {
            let x_scale = plot_width / (xaxis.limits.1 - xaxis.limits.0);
            ((x - xaxis.limits.0) * x_scale) + yaxis.margin
        };

        let to_canvas_y = |y| {
            let y_scale = plot_height / (yaxis.limits.1 - yaxis.limits.0);
            ((y - yaxis.limits.0) * y_scale) + xaxis.margin
        };

        self.draw_axes(&xaxis, &yaxis, to_canvas_x, to_canvas_y);

        // Draw the data series
        if !x_values.is_empty() {
            self.pdf
                .set_clipping_box(
                    Point {
                        x: to_canvas_x(xaxis.limits.0) - 2.0,
                        y: to_canvas_y(yaxis.limits.0) - 2.0,
                    },
                    Size {
                        width: to_canvas_x(xaxis.limits.1) - to_canvas_x(xaxis.limits.0) + 4.0,
                        height: to_canvas_y(yaxis.limits.1) - to_canvas_y(yaxis.limits.0) + 4.0,
                    },
                )
                .set_line_width(1.5)
                .set_color(Color {
                    red: 31,
                    green: 119,
                    blue: 180,
                })
                .draw_line(
                    x_values.iter().map(|&v| to_canvas_x(v)),
                    y_values.iter().map(|&v| to_canvas_y(v)),
                )
                .set_color(Color::gray(0));
        }

        self
    }

    pub fn image(
        &mut self,
        image_data: &[f64],
        image_width: usize,
        image_height: usize,
    ) -> &mut Self {
        // Convert the image to u8 and apply a color map
        assert!(image_width * image_height == image_data.len());

        let mut png_bytes = Vec::with_capacity(image_data.len() * 3);
        let mut max = std::f64::MIN;
        let mut min = std::f64::MAX;
        for i in image_data
            .iter()
            .filter(|i| !i.is_nan() && !i.is_infinite())
        {
            if *i < min {
                min = *i;
            }
            if *i > max {
                max = *i;
            }
        }

        let map = colormaps::VIRIDIS;
        for i in image_data {
            if i.is_nan() || i.is_infinite() {
                png_bytes.extend(&[255, 255, 255]);
            } else {
                let i = i.max(min); // upper-end clipping is applied by the line below
                let index = ((i - min) / (max - min) * 255.0) as usize;
                png_bytes.push((map[index][0] * 255.0) as u8);
                png_bytes.push((map[index][1] * 255.0) as u8);
                png_bytes.push((map[index][2] * 255.0) as u8);
            }
        }

        let (xaxis, yaxis) = self.digest_tick_settings(&[], &[]);

        let width = self.width;
        let height = self.height;

        let plot_width =
            width - yaxis.margin - self.pdf.width_of(xaxis.tick_labels.last().unwrap());
        let plot_height = height - xaxis.margin - self.font_size;
        let plot_size = plot_width.min(plot_height);

        // This is a hack; we adjust the height and width so that the generated PDF file has its
        // dimensions adjusted
        // TODO: This change should be ephemeral
        self.height = plot_size + xaxis.margin + self.font_size;
        self.width = plot_size + yaxis.margin + self.font_size;

        // Function to convert from plot pixels to canvas pixels
        let to_canvas_x = |x| {
            let x_scale = plot_size / (xaxis.limits.1 - xaxis.limits.0);
            ((x - xaxis.limits.0) * x_scale) + yaxis.margin
        };

        let to_canvas_y = |y| {
            let y_scale = plot_size / (yaxis.limits.1 - yaxis.limits.0);
            ((y - yaxis.limits.0) * y_scale) + xaxis.margin
        };

        self.draw_axes(&xaxis, &yaxis, to_canvas_x, to_canvas_y);

        let x_extent = to_canvas_x(xaxis.limits.1) - to_canvas_x(xaxis.limits.0) - 1.0;
        let y_extent = to_canvas_y(yaxis.limits.1) - to_canvas_y(yaxis.limits.0) - 1.0;
        self.pdf.transform(
            Matrix::scale(
                x_extent / (image_width as f64),
                y_extent / (image_width as f64),
            ) * Matrix::translate(
                to_canvas_x(xaxis.limits.0) + 0.5,
                to_canvas_y(yaxis.limits.0) + 0.5,
            ),
        );
        self.pdf.add_image_at(
            pdfpdf::Image::new(&png_bytes, image_width as u64, image_height as u64),
            pdfpdf::Point { x: 0, y: 0 },
        );
        self
    }

    pub fn write_to<F>(&mut self, filename: F) -> std::io::Result<()>
    where
        F: AsRef<std::path::Path>,
    {
        self.pdf.write_to(filename)
    }
}
