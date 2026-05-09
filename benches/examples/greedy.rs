use benches::{black_box as bb, print_header, print_row, time, want};
use fancy_regex::Regex as FancyRe;
use pcre2::bytes::RegexBuilder as PcreBuild;
use regex::Regex as Regex1;
use resharp::{Regex as Resharp, RegexOptions};

struct Case {
    title: &'static str,
    so: &'static str,
    rs: &'static str,
    inputs: &'static [(&'static str, fn() -> String)],
    code: fn(&str) -> usize,
}

fn pad200() -> String { "abcdefghij".repeat(20) }

fn end_short_present() -> String { format!("{}END tail", pad200()) }
fn end_long_present() -> String { format!("{}END tail", pad200().repeat(20)) }
fn end_long_absent() -> String { pad200().repeat(20) }
fn end_dense_partial() -> String { "abcEefghij".repeat(400) }

fn br_well_formed() -> String { "[abcdefgh]".repeat(50) }
fn br_unclosed() -> String { "[".repeat(500) }
fn br_mixed() -> String {
    let mut s = String::new();
    for i in 0..50 {
        if i % 10 == 9 { s.push_str("[unclosed_chunk_"); s.push_str(&i.to_string()); s.push(' '); }
        else { s.push_str("[okay_entry_"); s.push_str(&i.to_string()); s.push(']'); }
    }
    s
}

fn count_end(s: &str) -> usize {
    let mut n = 0;
    let mut rest = s;
    while let Some(i) = rest.find("END") { n += 1; rest = &rest[i + 3..]; }
    n
}

fn count_brackets(s: &str) -> usize {
    let b = s.as_bytes();
    let (mut i, mut n) = (0, 0);
    while let Some(lb) = b[i..].iter().position(|&c| c == b'[') {
        let lb = i + lb;
        match b[lb + 1..].iter().position(|&c| c == b']') {
            Some(rb) => { n += 1; i = lb + 1 + rb + 1; }
            None => break,
        }
    }
    n
}

const CASES: &[Case] = &[
    Case {
        title: "Q7124778: anything up to sequence `END` (find_all count)",
        so: r".*?END",
        rs: r"~(_*END_*)END",
        inputs: &[
            ("short_present", end_short_present),
            ("long_present",  end_long_present),
            ("long_absent",   end_long_absent),
            ("dense_partial", end_dense_partial),
        ],
        code: count_end,
    },
    Case {
        title: "Q6109882: between `[` and `]` (find_all count)",
        so: r"\[(.*?)\]",
        rs: r"\[(~(_*\]_*))\]",
        inputs: &[
            ("well_formed", br_well_formed),
            ("unclosed",    br_unclosed),
            ("mixed",       br_mixed),
        ],
        code: count_brackets,
    },
];

fn run(c: &Case) {
    if !want(c.title) { return; }
    println!("\n# {}\n", c.title);
    println!("- SO answer: `{}`", c.so);
    println!("- RE#:       `{}`", c.rs);

    let f = FancyRe::new(c.so).unwrap();
    let r = Regex1::new(c.so).unwrap();
    let p = PcreBuild::new().build(c.so).unwrap();
    let rs = Resharp::with_options(c.rs, RegexOptions::default().multiline(false)).unwrap();

    print_header("results", &["fancy", "regex", "pcre2", "resharp", "code"]);
    for (name, mk) in c.inputs {
        let s = mk();
        let b = s.as_bytes();
        let tf = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            time(|| { bb(f.find_iter(bb(&s)).count()); })
        })) { Ok(v) => v, Err(_) => f64::NAN };
        let tr = time(|| { bb(r.find_iter(bb(&s)).count()); });
        let tp = time(|| { bb(p.find_iter(bb(b)).count()); });
        let trs = time(|| { bb(rs.find_all(bb(b)).unwrap().len()); });
        let tc = time(|| { bb((c.code)(bb(&s))); });
        print_row(name, &[tf, tr, tp, trs, tc]);
    }
}

fn main() { for c in CASES { run(c); } }
