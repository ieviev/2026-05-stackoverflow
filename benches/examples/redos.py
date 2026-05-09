import re, regex, time, sys

def pad(n): return "ciao mandi " * n

cases_sol = [
    ("match_1kb",    lambda: pad(46)   + "sol " + pad(46)),
    ("match_10kb",   lambda: pad(455)  + "sol " + pad(455)),
    ("match_100kb",  lambda: pad(4545) + "sol " + pad(4545)),
    ("nomatch_1kb",  lambda: pad(46)   + "sal " + pad(46)),
    ("nomatch_10kb", lambda: pad(455)  + "sal " + pad(455)),
    ("nomatch_100kb",lambda: pad(4545) + "sal " + pad(4545)),
]
cases_csv = [
    ("match",        lambda: "a,a,a,a,a,a,a,a,a,a,P"),
    ("commas_20",    lambda: "," * 20),
    ("commas_25",    lambda: "," * 25),
    ("commas_30",    lambda: "," * 30),
]
cases_fb = [
    ("match_1kb",     lambda: pad(30) + "foo" + pad(30) + "bar" + pad(30)),
    ("match_10kb",    lambda: pad(303) + "foo" + pad(303) + "bar" + pad(303)),
    ("nomatch_1kb",   lambda: pad(46) + "foo" + pad(46)),
    ("nomatch_10kb",  lambda: pad(455) + "foo" + pad(455)),
    ("nomatch_100kb", lambda: pad(4545)+ "foo" + pad(4545)),
]
cases_ns = [
    ("nomatch_14", lambda: "a"*14 + "!"),
    ("nomatch_16", lambda: "a"*16 + "!"),
    ("nomatch_18", lambda: "a"*18 + "!"),
]
cases_aopt = [
    ("nomatch_15", lambda: "a"*15),
    ("nomatch_20", lambda: "a"*20),
    ("nomatch_25", lambda: "a"*25),
]

def bench(fn, budget=0.3, max_total=10.0):
    t0 = time.perf_counter(); fn(); one = max(time.perf_counter() - t0, 1e-9)
    chunk = max(1, min(1<<20, int(1e-3 / one)))
    warm_end = time.perf_counter() + 0.05
    while time.perf_counter() < warm_end:
        for _ in range(chunk): fn()
    start = time.perf_counter()
    deadline = start + budget
    hard = start + max_total
    iters = 0
    while time.perf_counter() < deadline and time.perf_counter() < hard:
        for _ in range(chunk): fn()
        iters += chunk
    return (time.perf_counter() - start) / iters * 1e9

def fmt(ns):
    if ns < 1000: return f"{ns:.1f} ns"
    if ns < 1e6:  return f"{ns/1e3:.2f} us"
    if ns < 1e9:  return f"{ns/1e6:.2f} ms"
    return f"{ns/1e9:.2f} s"

def run(title, pattern, cases, budget=0.3):
    print(f"\n## {title}\n\n- pattern: `{pattern}`\n")
    re_pat = re.compile(pattern)
    rx_pat = regex.compile(pattern)
    print(f"{'input':<16} {'python_re':>14} {'python_regex':>14}")
    for name, mk in cases:
        s = mk()
        t1 = bench(lambda: re_pat.search(s), budget=budget)
        t2 = bench(lambda: rx_pat.search(s), budget=budget)
        print(f"{name:<16} {fmt(t1):>14} {fmt(t2):>14}")

run("Q26214328: (.*)sol(.*)",                          r"(.*)sol(.*)",       cases_sol)
run("CSV nth field: ^(.*?,){10}P$",                    r"^(.*?,){10}P$",     cases_csv)
run(".*foo.*bar",                                      r".*foo.*bar",        cases_fb)
run("Q4495733: ((\\w+)(::)?)+",                        r"^((\w+)(::)?)+$",   cases_ns)
run("(a|a?)+b",                                        r"^(a|a?)+b$",        cases_aopt)
