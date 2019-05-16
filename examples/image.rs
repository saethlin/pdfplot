use pdfplot::Plot;

fn main() {
    let data = vec![0; 100 * 100 * 3];
    Plot::new()
        .xlabel("xlabel")
        .ylabel("ylabel")
        .ylim(0., 0.05)
        .xlim(150.0, 500.0)
        .tick_length(10.0)
        .x_tick_interval(50.)
        .y_tick_interval(0.008)
        .plot(&[], &[], Some((&data, 100, 100)))
        .write_to("empty.pdf")
        .unwrap();
}
