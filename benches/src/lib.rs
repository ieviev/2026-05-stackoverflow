// Tiny bench harness. Each cell: warm 50ms, measure ~300ms, return ns/iter.
// Rows are normalized so the fastest cell in the row = 1.00x.
//
// CLI:
//   <title-substr>...        run sections whose title contains any substring
//   -e REGEX | --engines=RE  run only columns whose name matches REGEX;
//                            disabled cells are skipped (no work done) and
//                            omitted from the printed table.
// Convention: examples must call `time()` once per column in the same order
// as the `cols` slice passed to `print_header`.

use std::cell::{Cell, RefCell};
use std::hint::black_box as bb;
use std::time::{Duration, Instant};

pub use std::hint::black_box;

pub fn sink_usize(acc: &mut usize, v: usize) {
    *acc ^= black_box(v);
}

thread_local! {
    static MASK: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
    static IDX:  Cell<usize>        = const { Cell::new(0) };
}

fn engine_filter() -> Option<regex::Regex> {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if let Some(p) = a.strip_prefix("--engines=") {
            return Some(regex::Regex::new(p).expect("bad --engines regex"));
        }
        if a == "-e" || a == "--engines" {
            let p = args.next().expect("-e needs an argument");
            return Some(regex::Regex::new(&p).expect("bad -e regex"));
        }
    }
    None
}

fn title_args() -> Vec<String> {
    let mut out = Vec::new();
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a.starts_with("--engines=") { continue; }
        if a == "-e" || a == "--engines" { let _ = args.next(); continue; }
        out.push(a);
    }
    out
}

pub fn time(mut f: impl FnMut()) -> f64 {
    let idx = IDX.with(|i| { let v = i.get(); i.set(v + 1); v });
    let enabled = MASK.with(|m| m.borrow().get(idx).copied().unwrap_or(true));
    if !enabled { return f64::NAN; }

    let t0 = Instant::now();
    f();
    let one = t0.elapsed().as_secs_f64().max(1e-9);
    let chunk = ((1e-3 / one) as u64).clamp(1, 1 << 20);

    let warm_end = Instant::now() + Duration::from_millis(50);
    while Instant::now() < warm_end {
        for _ in 0..chunk { f(); }
    }

    let start = Instant::now();
    let deadline = start + Duration::from_millis(300);
    let mut iters = 0u64;
    while Instant::now() < deadline {
        for _ in 0..chunk { f(); }
        iters += chunk;
    }
    bb(start.elapsed().as_secs_f64() * 1e9 / iters as f64)
}

pub fn time_fallible<F, R, E>(f: &F) -> f64
where
    F: Fn() -> Result<R, E>,
{
    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut hit = false;
        let t = time(|| match f() {
            Ok(v) => { bb(v); }
            Err(_) => { hit = true; }
        });
        if hit { f64::NAN } else { t }
    })) {
        Ok(v) => v,
        Err(_) => f64::NAN,
    }
}

pub fn want(title: &str) -> bool {
    let args = title_args();
    if args.is_empty() { return true; }
    args.iter().any(|a| title.contains(a.as_str()))
}

pub fn print_header(title: &str, cols: &[&str]) {
    let re = engine_filter();
    let mask: Vec<bool> = cols.iter()
        .map(|c| re.as_ref().map_or(true, |r| r.is_match(c)))
        .collect();
    MASK.with(|m| *m.borrow_mut() = mask.clone());
    IDX.with(|i| i.set(0));

    println!("\n## {title}\n");
    print!("| input |");
    for (c, &on) in cols.iter().zip(&mask) {
        if on { print!(" {c} |"); }
    }
    println!();
    print!("|---|");
    for &on in &mask {
        if on { print!("--:|"); }
    }
    println!();
}

pub fn print_row(name: &str, ns: &[f64]) {
    IDX.with(|i| i.set(0));
    let mask = MASK.with(|m| m.borrow().clone());
    let kept: Vec<f64> = ns.iter().enumerate()
        .filter(|(i, _)| mask.get(*i).copied().unwrap_or(true))
        .map(|(_, t)| *t)
        .collect();
    print!("| `{name}` |");
    let base = kept.iter().cloned().filter(|t| t.is_finite()).fold(f64::INFINITY, f64::min);
    for &t in &kept {
        if !t.is_finite() { print!(" FAIL |"); continue; }
        let b = if t == base { "**" } else { "" };
        if t < 1000.0 {
            print!(" {b}{t:.1} ns ({:.2}x){b} |", t / base);
        } else if t < 1_000_000.0 {
            print!(" {b}{:.2} us ({:.2}x){b} |", t / 1e3, t / base);
        } else if t < 1_000_000_000.0 {
            print!(" {b}{:.2} ms ({:.2}x){b} |", t / 1e6, t / base);
        } else {
            print!(" {b}{:.2} s ({:.2}x){b} |", t / 1e9, t / base);
        }
    }
    println!();
}
