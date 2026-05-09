import re, time

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

def pad200(): return "abcdefghij" * 20

print("\n## Q7124778: .*?END (find_all count)\n")
pat = re.compile(r".*?END")
cases = [
    ("short_present", lambda: pad200() + "END tail"),
    ("long_present",  lambda: pad200() * 20 + "END tail"),
    ("long_absent",   lambda: pad200() * 20),
    ("dense_partial", lambda: "abcEefghij" * 400),
]
print(f"{'input':<20} {'python_re':>14}")
for name, mk in cases:
    s = mk()
    t = bench(lambda: len(pat.findall(s)))
    print(f"{name:<20} {fmt(t):>14}")

print("\n## Q6109882: between [ and ] (find_all count)\n")
pat2 = re.compile(r"\[(.*?)\]")
cases2 = [
    ("well_formed", lambda: "[abcdefgh]" * 50),
    ("unclosed",    lambda: "[" * 500),
    ("mixed",       lambda: "".join(
        f"[unclosed_chunk_{i} " if i % 10 == 9 else f"[okay_entry_{i}]"
        for i in range(50)
    )),
]
print(f"{'input':<20} {'python_re':>14}")
for name, mk in cases2:
    s = mk()
    t = bench(lambda: len(pat2.findall(s)))
    print(f"{name:<20} {fmt(t):>14}")

print("\n## Q7167279: 5 groups between <div></div>\n")
pat3 = re.compile(r"^<div>(.*?)</div><div>(.*?)</div><div>(.*?)</div><div>(.*?)</div><div>(.*?)</div>$")
def tags_match():
    s = ""
    for i in range(5): s += f"<div>cell_{i}</div>"
    return s
def tags_amb_25():
    s = "<div>"
    for i in range(25): s += f"x{i}</div><div>"
    s += "end"
    return s
cases3 = [
    ("match_5_pairs",    tags_match),
    ("nonmatch_25_delims", tags_amb_25),
]
print(f"{'input':<24} {'python_re':>14}")
for name, mk in cases3:
    s = mk()
    t = bench(lambda: pat3.search(s), budget=0.3, max_total=10.0)
    print(f"{name:<24} {fmt(t):>14}")
