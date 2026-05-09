use benches::{black_box as bb, print_header, print_row, time, want};

const PAT: &str = r"(?<=From:.*)alice";

fn make_input(n: usize) -> String {
    format!("From: alice@example.com\n{}", "some log line with data here\n".repeat(n))
}

fn count_regress(re: &regress::Regex, input: &str) -> usize {
    let mut count = 0;
    let mut it = re.find_iter(input);
    while let Some(_) = it.next() { count += 1; }
    count
}

fn main() {
    let title = "lookbehind: (?<=From:.*)alice";
    if !want(title) { return; }

    let regress_re = regress::Regex::new(PAT).unwrap();

    let dbg_input = make_input(5);
    eprintln!("regress: {} matches", count_regress(&regress_re, &dbg_input));

    print_header(title, &["regress(js)"]);
    for n in [50, 200, 500, 1000, 2000] {
        let input = make_input(n);

        let tr = time(|| { bb(count_regress(&regress_re, bb(&input))); });

        print_row(&format!("{n} lines ({} B)", input.len()), &[tr]);
    }
}
