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
