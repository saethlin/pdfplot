pub fn loadtxt(filename: &str) -> Vec<Vec<f64>> {
    let mut columns = Vec::new();
    for line in std::fs::read_to_string(filename).unwrap().lines() {
        for (w, word) in line.split_whitespace().enumerate() {
            if columns.len() <= w {
                columns.push(Vec::new());
            }
            columns[w].push(word.parse::<f64>().unwrap());
        }
    }

    columns
}

pub(crate) trait ToU64 {
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

pub(crate) trait FloatMax {
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
