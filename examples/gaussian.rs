use ndarray::Array;
use pdfplot::Plot;

fn main() {
    let x = Array::linspace(0.0f64, 600.0, 4096);
    let y = x.map(|x| (-(x - 300.0).powi(2) / 1200.0).exp() * 600.0);
    Plot::new()
        .ylim(-100.0, 600.0)
        .xlim(0.0, 600.0)
        .plot(x.as_slice().unwrap(), y.as_slice().unwrap())
        .write_to("plot.pdf")
        .unwrap();
}
