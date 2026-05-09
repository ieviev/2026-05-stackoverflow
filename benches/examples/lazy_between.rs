use benches::{black_box as bb, print_header, print_row, time, time_fallible, want};
use fancy_regex::RegexBuilder as FancyBuild;
use pcre2::bytes::RegexBuilder as PcreBuild;
use regex::Regex as Regex1;
use resharp::{Regex as Resharp, RegexOptions};

fn tags_match() -> String {
    let mut s = String::new();
    for i in 0..5 { s.push_str(&format!("<div>cell_{i}</div>")); }
    s
}

fn tags_amb_25() -> String {
    let mut s = String::from("<div>");
    for i in 0..25 { s.push_str(&format!("x{i}</div><div>")); }
    s.push_str("end");
    s
}

fn main() {
    let title = "Q7167279: 5 groups between <div></div>";
    if !want(title) { return; }
    println!("\n# {title}\n");

    let pat = r"^<div>(.*?)</div><div>(.*?)</div><div>(.*?)</div><div>(.*?)</div><div>(.*?)</div>$";
    let rs_pat = r"^<div>(~(_*</div>_*))</div><div>(~(_*</div>_*))</div><div>(~(_*</div>_*))</div><div>(~(_*</div>_*))</div><div>(~(_*</div>_*))</div>$";

    let f = FancyBuild::new(pat).backtrack_limit(50_000_000).build().ok();
    let r = Regex1::new(pat).ok();
    let p = PcreBuild::new().build(pat).ok();
    let rs = Resharp::with_options(rs_pat, RegexOptions::default().multiline(false)).ok();

    print_header("results", &["fancy-regex", "regex", "pcre2", "resharp"]);

    let cases: &[(&str, fn() -> String)] = &[
        ("match, 5 pairs", tags_match),
        ("non-match, 25 delims", tags_amb_25),
    ];

    for (name, mk) in cases {
        let s = mk();
        let b = s.as_bytes();

        let tf = f.as_ref().map_or(f64::NAN, |re| time_fallible(&|| re.find(bb(&s))));
        let tr = r.as_ref().map_or(f64::NAN, |re| time(|| { bb(re.find(bb(&s))); }));
        let tp = p.as_ref().map_or(f64::NAN, |re| time_fallible(&|| re.find(bb(b))));
        let trs = rs.as_ref().map_or(f64::NAN, |re| time(|| { bb(re.is_match(bb(b)).unwrap()); }));

        print_row(name, &[tf, tr, tp, trs]);
    }
}
