use ndarray::Array;
use pdfplot::Plot;

fn main() {
    let x = Array::linspace(10.0f64, 590.0, 4096);
    let y = x.map(|x| (-(x - 300.0).powi(2) / 1200.0).exp() * 590.0 + 0.1 * x);
    Plot::new()
        .xlabel("xlabel")
        .ylabel("ylabel")
        .plot(x.as_slice().unwrap(), y.as_slice().unwrap())
        .write_to("plot.pdf")
        .unwrap();
}
