// Complement cluster on stackoverflow. Four structurally distinct shapes:
//   Q406230   "line does NOT contain word"           5.5M views
//   Q16398471 "string does NOT end with suffix"      376k views (lookbehind)
//   Q2116328  "string does NOT start with prefix"    366k views (parity case)
//   Q1687620  "line does NOT contain a regex"        940k views (composition)

use std::sync::OnceLock;

use benches::{black_box as bb, print_header, print_row, time, want};
use fancy_regex::Regex as FancyRe;
use pcre2::bytes::{Regex as PcreRe, RegexBuilder as PcreBuild};
use resharp::{Regex as Resharp, RegexOptions};

const LINE: &str = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore";
const SIZES: &[(&str, usize)] = &[("10_lines", 10), ("100_lines", 100)];

#[derive(Clone, Copy)]
enum Inject { None, First, Last }

struct Case {
    title: &'static str,
    forbidden_re:  &'static str,
    forbidden_lit: &'static str,
    answers: &'static [(&'static str, &'static str)],
    resharp: &'static str,
    injects: &'static [(&'static str, Inject)],
    code:    fn(&str) -> bool,
}

fn rs(p: &str) -> Resharp {
    Resharp::with_options(p, RegexOptions::default().multiline(false)).unwrap()
}
fn fc(p: &str) -> FancyRe { FancyRe::new(p).unwrap() }
fn pc(p: &str) -> PcreRe  { PcreBuild::new().build(p).unwrap() }

fn build_inputs(token: &str, injects: &[(&str, Inject)]) -> Vec<(String, Vec<String>)> {
    let mut out = Vec::new();
    for (sz, n) in SIZES {
        for (inj_name, inj) in injects {
            let lines: Vec<String> = (0..*n).map(|i| {
                let dirty = match inj {
                    Inject::None  => false,
                    Inject::First => i == 0,
                    Inject::Last  => i == n - 1,
                };
                if dirty { format!("{token} {LINE}") } else { LINE.to_string() }
            }).collect();
            out.push((format!("{sz}_{inj_name}"), lines));
        }
    }
    out
}

fn run_case(c: &Case) {
    let r = rs(&c.resharp.replace("{T}", c.forbidden_re));
    let inputs = build_inputs(c.forbidden_lit, c.injects);

    for (label, tpl) in c.answers {
        let pat = tpl.replace("{T}", c.forbidden_re);
        let f = fc(&pat);
        let p = pc(&pat);
        print_header(
            &format!("{} pattern={label}", c.title),
            &["fancy", "pcre2", "resharp"],
        );
        for (name, lines) in &inputs {
            let line_bytes: Vec<&[u8]> = lines.iter().map(|l| l.as_bytes()).collect();
            let tf = time(|| {
                let mut acc = 0usize;
                for l in lines { if f.is_match(bb(l)).unwrap() { acc += 1; } }
                bb(acc);
            });
            let tp = time(|| {
                let mut acc = 0usize;
                for &b in &line_bytes { if p.is_match(bb(b)).unwrap() { acc += 1; } }
                bb(acc);
            });
            let tr = time(|| {
                let mut acc = 0usize;
                for &b in &line_bytes { if r.is_match(bb(b)).unwrap() { acc += 1; } }
                bb(acc);
            });
            print_row(name, &[tf, tp, tr]);
        }
    }
}

const CASES: &[Case] = &[
    Case {
        title: "Q406230: line does NOT contain word",
        forbidden_re:  "FORBIDDEN",
        forbidden_lit: "FORBIDDEN",
        answers: &[
            ("tempered  `^((?!W).)*$`", r"^((?!{T}).)*$"),
            ("lookahead `^(?!.*W).*$`", r"^(?!.*{T}).*$"),
        ],
        resharp: r"^.*$&~(_*{T}_*)",
        injects: &[("nomatch", Inject::None), ("match_first", Inject::First), ("match_last", Inject::Last)],
        code: |s| !s.contains("FORBIDDEN"),
    },
    Case {
        title: "Q16398471: string does NOT end with suffix",
        forbidden_re:  "SUFFIX",
        forbidden_lit: "SUFFIX",
        answers: &[
            ("lookbehind `.*(?<!S)$`",   r"(?s).*(?<!{T})$"),
            ("lookahead  `^(?!.*S$).*$`", r"^(?!.*{T}$).*$"),
        ],
        resharp: r"^_*$&~(_*{T})",
        injects: &[("nomatch", Inject::None), ("match_last", Inject::Last)],
        code: |s| !s.ends_with("SUFFIX"),
    },
    Case {
        title: "Q2116328/Q899422: string does NOT start with prefix",
        forbidden_re:  "PREFIX",
        forbidden_lit: "PREFIX",
        answers: &[
            ("lookahead `^(?!P).*`", r"(?s)^(?!{T}).*"),
        ],
        resharp: r"^.*$&~({T}_*)",
        injects: &[("nomatch", Inject::None), ("match_first", Inject::First)],
        code: |s| !s.starts_with("PREFIX"),
    },
    Case {
        title: r"Q1687620: line does NOT contain regex `index\.php\?id=\d+`",
        forbidden_re:  r"index\.php\?id=\d+",
        forbidden_lit: "index.php?id=42",
        answers: &[
            ("tempered  `^((?!P).)*$`", r"^((?!{T}).)*$"),
            ("lookahead `^(?!.*P).*$`", r"^(?!.*{T}).*$"),
        ],
        resharp: r"^.*$&~(_*{T}_*)",
        injects: &[("nomatch", Inject::None), ("match_last", Inject::Last)],
        code: |s| {
            static R: OnceLock<regex::Regex> = OnceLock::new();
            !R.get_or_init(|| regex::Regex::new(r"index\.php\?id=\d+").unwrap()).is_match(s)
        },
    },
];

fn run_lookbehind_scaling() {
    if !want("lookbehind scaling") { return; }
    let pat = r"(?s).*(?<!SUFFIX)$";
    let f = fc(pat);
    let p = pc(pat);
    let r = rs(r"^_*$&~(_*SUFFIX)");
    let pad = "lorem ipsum dolor sit amet consectetur adipiscing elit sed do eiusmod tempor incididunt ut labore ";
    print_header(
        "Q16398471 scaling: `.*(?<!SUFFIX)$` on one long line ending with SUFFIX",
        &["fancy", "pcre2", "resharp"],
    );
    for n in [5] {
        let s = format!("{}SUFFIX", pad.repeat(n));
        let b = s.as_bytes();
        let len = s.len();
        let tf = time(|| { bb(f.is_match(bb(&s)).unwrap()); });
        let tp = time(|| { bb(p.is_match(bb(b)).unwrap()); });
        let tr = time(|| { bb(r.is_match(bb(b)).unwrap()); });
        print_row(&format!("{n} lines ({len} B)"), &[tf, tp, tr]);
    }
}

fn main() {
    if want("negation cluster") {
        for c in CASES { run_case(c); }
    }
    run_lookbehind_scaling();
}
