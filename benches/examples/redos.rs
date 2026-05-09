use benches::{black_box as bb, print_header, print_row, time, time_fallible, want};
use fancy_regex::RegexBuilder as FancyBuild;
use pcre2::bytes::RegexBuilder as PcreBuild;
use resharp::{Regex as Resharp, RegexOptions};

struct Case {
    title: &'static str,
    pattern: &'static str,
    rs_pattern: &'static str,
    inputs: &'static [(&'static str, fn() -> String)],
}

fn pad(n: usize) -> String {
    "ciao mandi ".repeat(n)
}

fn match_1kb() -> String {
    format!("{}sol {}", pad(46), pad(46))
}
fn match_10kb() -> String {
    format!("{}sol {}", pad(455), pad(455))
}
fn match_100kb() -> String {
    format!("{}sol {}", pad(4545), pad(4545))
}

fn nomatch_1kb() -> String {
    format!("{}sal {}", pad(46), pad(46))
}
fn nomatch_10kb() -> String {
    format!("{}sal {}", pad(455), pad(455))
}
fn nomatch_100kb() -> String {
    format!("{}sal {}", pad(4545), pad(4545))
}

fn csv_match() -> String {
    "a,a,a,a,a,a,a,a,a,a,P".to_string()
}
fn csv_commas_20() -> String {
    ",".repeat(20)
}
fn csv_commas_25() -> String {
    ",".repeat(25)
}
fn csv_commas_30() -> String {
    ",".repeat(30)
}

fn fb_match_1kb() -> String {
    format!("{}foo{}bar{}", pad(30), pad(30), pad(30))
}
fn fb_match_10kb() -> String {
    format!("{}foo{}bar{}", pad(303), pad(303), pad(303))
}
fn fb_nomatch_1kb() -> String {
    format!("{}foo{}", pad(46), pad(46))
}
fn fb_nomatch_10kb() -> String {
    format!("{}foo{}", pad(455), pad(455))
}
fn fb_nomatch_100kb() -> String {
    format!("{}foo{}", pad(4545), pad(4545))
}

fn ns_14() -> String {
    format!("{}!", "a".repeat(14))
}
fn ns_16() -> String {
    format!("{}!", "a".repeat(16))
}
fn ns_18() -> String {
    format!("{}!", "a".repeat(18))
}

fn aopt_15() -> String {
    "a".repeat(15)
}
fn aopt_20() -> String {
    "a".repeat(20)
}
fn aopt_25() -> String {
    "a".repeat(25)
}

const CASES: &[Case] = &[
    Case {
        title: "Q26214328: `(.*)sol(.*)` innocent pattern, quadratic on no-match",
        pattern: r"(.*)sol(.*)",
        rs_pattern: r"(.*)sol(.*)",
        inputs: &[
            ("match_1kb", match_1kb),
            ("match_10kb", match_10kb),
            ("match_100kb", match_100kb),
            ("nomatch_1kb", nomatch_1kb),
            ("nomatch_10kb", nomatch_10kb),
            ("nomatch_100kb", nomatch_100kb),
        ],
    },
    Case {
        title: "CSV nth field: `^(.*?,){10}P$` lazy quantifier doesn't save you on no-match",
        pattern: r"^(.*?,){10}P$",
        rs_pattern: "",
        inputs: &[
            ("match", csv_match),
            ("commas_20", csv_commas_20),
            ("commas_25", csv_commas_25),
            ("commas_30", csv_commas_30),
        ],
    },
    Case {
        title: "`.*foo.*bar` two-literal cousin of `(.*)X(.*)`, quadratic on no-match",
        pattern: r".*foo.*bar",
        rs_pattern: r".*foo.*bar",
        inputs: &[
            ("match_1kb", fb_match_1kb),
            ("match_10kb", fb_match_10kb),
            ("nomatch_1kb", fb_nomatch_1kb),
            ("nomatch_10kb", fb_nomatch_10kb),
            ("nomatch_100kb", fb_nomatch_100kb),
        ],
    },
    Case {
        title: "Q4495733: `((\\w+)(::)?)+` namespace parser, exponential on no-match",
        pattern: r"^((\w+)(::)?)+$",
        rs_pattern: r"^((\w+)(::)?)+$",
        inputs: &[
            ("nomatch_14", ns_14),
            ("nomatch_16", ns_16),
            ("nomatch_18", ns_18),
        ],
    },
    Case {
        title: "`(a|a?)+b` `?` inside `+` is the trap, exponential on no-match",
        pattern: r"^(a|a?)+b$",
        rs_pattern: r"^(a|a?)+b$",
        inputs: &[
            ("nomatch_15", aopt_15),
            ("nomatch_20", aopt_20),
            ("nomatch_25", aopt_25),
        ],
    },
    Case {
        title:
            "Q26214328 fix attempt: `(.*?)sol(.*)` the SO-recommended `lazy first` workaround",
        pattern: r"(.*?)sol(.*)",
        rs_pattern: r"(.*?)sol(.*)",
        inputs: &[("match_10kb", match_10kb), ("nomatch_10kb", nomatch_10kb)],
    },
];

fn run(c: &Case) {
    if !want(c.title) {
        return;
    }
    println!("\n## {}\n", c.title);
    println!("- pattern: `{}`\n", c.pattern);

    let f = FancyBuild::new(c.pattern)
        .backtrack_limit(50_000_000)
        .build()
        .ok();
    let p = PcreBuild::new().build(c.pattern).ok();
    let r = regex::Regex::new(c.pattern).ok();
    let rs = if c.rs_pattern.is_empty() { None } else {
        Resharp::with_options(c.rs_pattern, RegexOptions::default().multiline(false)).ok()
    };

    let has_rs = rs.is_some();
    if has_rs {
        print_header("results", &["fancy", "regex", "pcre2", "resharp"]);
    } else {
        print_header("results", &["fancy", "regex", "pcre2"]);
    }
    for (name, mk) in c.inputs {
        let s = mk();
        let b = s.as_bytes();
        let tf = f.as_ref().map_or(f64::NAN, |re| time_fallible(&|| re.find(bb(&s))));
        let tr = r.as_ref().map_or(f64::NAN, |re| time(|| { bb(re.find(bb(&s))); }));
        let tp = p.as_ref().map_or(f64::NAN, |re| time_fallible(&|| re.find(bb(b))));
        if has_rs {
            let trs = time(|| { bb(rs.as_ref().unwrap().is_match(bb(b)).unwrap()); });
            print_row(name, &[tf, tr, tp, trs]);
        } else {
            print_row(name, &[tf, tr, tp]);
        }
    }
}

fn main() {
    for c in CASES {
        run(c);
    }
}
