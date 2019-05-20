use pdfplot::Plot;

fn main() {
    let mut data = vec![0.0; 100 * 100];
    data[5050] = 1.0;

    Plot::new()
        .xlabel("escape fraction")
        .ylabel("ylabel")
        .ylim(0., 0.05)
        .xlim(0.0, 1.0)
        .tick_length(10.0)
        .x_tick_interval(0.2)
        .y_tick_interval(0.008)
        .image(&data, 100, 100)
        .write_to("image.pdf")
        .unwrap();
}
