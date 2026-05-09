import regex, sys, time
input_str = open(sys.argv[1]).read()
iters = int(sys.argv[2])
pat = regex.compile(sys.argv[3])
n = len(pat.findall(input_str))
print(f"python: {n} matches", file=sys.stderr)
t0 = time.perf_counter_ns()
for _ in range(iters):
    pat.findall(input_str)
print((time.perf_counter_ns() - t0) // iters)
