// AND cluster on stackoverflow. The canonical answer in every one of these
// threads is chained `(?=.*X)` lookaheads. Each lookahead does an O(n) scan,
// so N conjuncts = N passes over the input. RE# expresses the same thing
// natively with `&` (intersection) and runs the conjunction in one DFA pass.
//
//   Q1559751   password: lower & upper & digit & symbol & len>=8   304k views
//   Q4389644   two words in any order                              242k views
//   Q2686147   two terms across newlines                           188k views
//   Q13911053  N-word search query (scaling)                       --

use benches::{black_box as bb, print_header, print_row, time, want};
use fancy_regex::Regex as FancyRe;
use pcre2::bytes::RegexBuilder as PcreBuild;
use resharp::{Regex as Resharp, RegexOptions};

struct Case {
    title:   String,
    look:    String,
    rs:      String,
    code:    Box<dyn Fn(&str) -> bool>,
    inputs:  Vec<(String, String)>,
}

fn run(c: &Case) {
    let f = FancyRe::new(&c.look).unwrap();
    let p = PcreBuild::new().build(&c.look).unwrap();
    let r = Resharp::with_options(&c.rs, RegexOptions::default().multiline(false)).unwrap();
    print_header(&c.title, &["fancy", "pcre2", "resharp", "code"]);
    for (name, s) in &c.inputs {
        let bytes = s.as_bytes();
        let tf = if f.is_match(s).is_ok()
            { time(|| { bb(f.is_match(bb(s)).unwrap()); }) } else { f64::NAN };
        let tp = if p.is_match(bytes).is_ok()
            { time(|| { bb(p.is_match(bb(bytes)).unwrap()); }) } else { f64::NAN };
        let tr = time(|| { bb(r.is_match(bb(bytes)).unwrap()); });
        let tc = time(|| { bb((c.code)(bb(s))); });
        print_row(name, &[tf, tp, tr, tc]);
    }
}

fn password() -> Case {
    Case {
        title: "Q1559751: password must contain lower, upper, digit, symbol, len>=8 (304k)".into(),
        look:  r"^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[!@#$%^&*]).{8,}$".into(),
        rs:    r"_*[a-z]_*&_*[A-Z]_*&_*[0-9]_*&_*[!@#$%\^&\*]_*&_{8,}".into(),
        code: Box::new(|s| {
            let (mut lo, mut up, mut di, mut sy) = (false, false, false, false);
            for &b in s.as_bytes() {
                lo |= b.is_ascii_lowercase();
                up |= b.is_ascii_uppercase();
                di |= b.is_ascii_digit();
                sy |= matches!(b, b'!'|b'@'|b'#'|b'$'|b'%'|b'^'|b'&'|b'*');
            }
            lo && up && di && sy && s.len() >= 8
        }),
        inputs: vec![
            ("short_valid".into(),  "Abcdefg1!".into()),
            ("short_reject".into(), "Abcdefg1".into()),
            ("long_valid".into(),
                format!("{}{}{}{}", "a".repeat(200), "B".repeat(200), "1".repeat(100), "!".repeat(100))),
            ("long_reject_no_digit".into(),
                format!("{}{}{}", "a".repeat(300), "B".repeat(300), "!".repeat(100))),
        ],
    }
}

fn two_terms() -> Case {
    let pad_small = "the quick brown fox jumps over the lazy dog ".repeat(20);   // ~880 B
    let pad_large = "the quick brown fox jumps over the lazy dog ".repeat(240);  // ~10 KB
    Case {
        title: "Q4389644/Q4487328: line contains 'jack' AND 'james' (242k+226k)".into(),
        look:  r"(?s)^(?=.*jack)(?=.*james).*$".into(),
        rs:    r"_*jack_*&_*james_*".into(),
        code:  Box::new(|s| s.contains("jack") && s.contains("james")),
        inputs: vec![
            ("small_both".into(),    format!("jack {pad_small} james")),
            ("small_only_jack".into(), format!("jack {pad_small}")),
            ("small_neither".into(),   pad_small.clone()),
            ("large_both".into(),    format!("jack {pad_large} james")),
            ("large_only_jack".into(), format!("jack {pad_large}")),
            ("large_neither".into(),   pad_large.clone()),
        ],
    }
}

fn n_words(n: usize) -> Case {
    const WORDS: &[&str] = &[
        "alpha","bravo","charlie","delta","echo","foxtrot","golf","hotel",
        "india","juliet","kilo","lima","mike","november","oscar","papa",
    ];
    let terms: Vec<&str> = WORDS.iter().take(n).copied().collect();
    let look = format!("(?s)^{}.*$",
        terms.iter().map(|w| format!("(?=.*{w})")).collect::<String>());
    let rs = terms.iter().map(|w| format!("_*{w}_*")).collect::<Vec<_>>().join("&");
    let owned: Vec<String> = terms.iter().map(|s| s.to_string()).collect();
    let pad = "lorem ipsum dolor sit amet consectetur\n".repeat(280); // ~11 KB
    let inputs = vec![
        ("all_present".into(),  format!("{pad}{}\n{pad}", terms.join(" "))),
        ("last_missing".into(), format!("{pad}{}\n{pad}", terms[..n-1].join(" "))),
        ("none_present".into(), pad.clone()),
    ];
    Case {
        title: format!("Q13911053: AND of {n} search terms across ~11 KB document"),
        look, rs,
        code: Box::new(move |s| owned.iter().all(|w| s.contains(w.as_str()))),
        inputs,
    }
}

fn main() {
    if !want("AND cluster") { return; }
    let cases = vec![password(), two_terms(), n_words(3), n_words(5), n_words(8)];
    for c in &cases { run(c); }
}
