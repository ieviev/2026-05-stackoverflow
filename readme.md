## benchmark experiments for blog post

benchmarks for [what 262,715 regex questions haven't answered](https://iev.ee/blog/what-262715-regex-questions-havent-answered/).

### dependencies

- rust, python 3, java (jdk), pcre2
- nix users: `nix develop`

### running

```
nix develop
./run.sh all
```

or run individual sections:

```
./run.sh complement
./run.sh intersection
./run.sh greedy
./run.sh redos
./run.sh lookaround
```

### results

output from benchmarks is written to `results/`.
