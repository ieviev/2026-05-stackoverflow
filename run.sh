#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/benches"

echo "=== Building all examples (release) ==="
cargo build --release --examples 2>&1 | tail -5

EXT="$(cargo metadata --format-version=1 --no-deps 2>/dev/null | sed -n 's/.*"target_directory":"\([^"]*\)".*/\1/p')/release/examples"

run_rust() {
    local name=$1; shift
    echo ""
    echo "--- $name ---"
    "$EXT/$name" "$@"
}

run_python() {
    local script=$1
    echo ""
    echo "--- python: $script ---"
    python3 "examples/$script"
}

run_java() {
    local class=$1; shift
    echo ""
    echo "--- java: $class ---"
    javac "examples/$class.java" -d examples/
    java -cp examples "$class" "$@"
}

section="${1:-all}"

case "$section" in
    complement|all)
        run_rust negation "negation cluster"
        ;;&
    intersection|all)
        run_rust and "AND cluster"
        ;;&
    greedy|matching-until|all)
        run_rust greedy
        run_rust lazy_between
        run_python greedy_py.py
        ;;&
    redos|performance|all)
        run_rust redos
        run_python redos.py
        ;;&
    lookaround|all)
        run_rust lookbehind_bench "lookbehind"
        run_java Lookbehind
        for n in 50 200 500 1000 2000; do
            tmp=$(mktemp)
            printf 'From: alice@example.com\n' > "$tmp"
            for i in $(seq 1 $n); do printf 'some log line with data here\n' >> "$tmp"; done
            iters=$( [ $n -le 500 ] && echo 200 || echo 20 )
            echo "python regex: n=$n ($(wc -c < "$tmp") B)"
            python3 examples/lookaround_bench.py "$tmp" $iters '(?<=From:.*)alice'
            rm "$tmp"
        done
        ;;&
    *)
        if [ "$section" = "all" ]; then
            exit 0
        fi
        if ! echo "complement intersection greedy matching-until redos performance lookaround all" | grep -qw "$section"; then
            echo "usage: $0 [complement|intersection|greedy|redos|lookaround|all]"
            exit 1
        fi
        ;;
esac
