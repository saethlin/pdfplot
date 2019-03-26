use pdfplot::Plot;

fn main() {
    let x = (0..4096)
        .map(|i| i as f64 / 4096. * 590. + 10.)
        .collect::<Vec<_>>();
    let y = x
        .iter()
        .map(|x| (-(x - 300.0).powi(2) / 1200.0).exp() * 0.06)
        .collect::<Vec<_>>();

    Plot::new()
        .xlabel("xlabel")
        .ylabel("ylabel")
        .plot(&x, &y)
        .write_to("gaussian.pdf")
        .unwrap();

    Plot::new()
        .xlabel("xlabel")
        .ylabel("ylabel")
        .ylim(0., 0.05)
        .xlim(150.0, 500.0)
        .tick_length(10.0)
        .x_tick_interval(50.)
        .y_tick_interval(0.008)
        .plot(&x, &y)
        .write_to("gaussian_full.pdf")
        .unwrap();
}
