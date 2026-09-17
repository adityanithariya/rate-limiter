# Default recipe: lists all available commands
default:
    @just --list

# Format check
fmt-check:
    cargo fmt --check

# Format code in-place
fmt:
    cargo fmt

# Run rate limiter criterion benchmarks
bench:
    cargo bench --bench rate_limiter_bench

bench-profile:
    cargo bench --bench rate_limiter_bench -- --profile-time 5

# Run all verification checks (format check, clippy, tests)
check: fmt-check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

# Profile a single algorithm (e.g. `just profile-mem fixed_window`)
profile-mem algo="fixed_window":
    @mkdir -p profiles
    cargo run --release --bin profile_memory -- {{ algo }}
    @mv dhat-heap.json profiles/dhat-{{ algo }}.json
    @echo "Saved profile to profiles/dhat-{{ algo }}.json"

# Profile all algorithms in sequence
profile-mem-all:
    @just profile-mem fixed_window
    @just profile-mem sliding_window_log
    @just profile-mem sliding_window_counter
    @just profile-mem token_bucket
    @just profile-mem leaky_bucket

# Print full recursive call traces for top memory allocation sites
report-mem file="dhat-heap.json":
    #!/usr/bin/env python3
    import json
    import sys
    from pathlib import Path

    path = Path("{{file}}")
    if not path.exists():
        print(f"File not found: {path}")
        sys.exit(1)

    try:
        with open(path) as f:
            d = json.load(f)
    except Exception as e:
        print(f"Error opening {path}: {e}")
        sys.exit(1)

    pps = d.get("pps", [])
    ftbl = d.get("ftbl", [])

    total_bytes = sum(p.get("tb", 0) for p in pps)
    total_blocks = sum(p.get("tbk", 0) for p in pps)
    peak_bytes = sum(p.get("mb", 0) for p in pps)
    leaked_bytes = sum(p.get("eb", 0) for p in pps)

    print("=" * 90)
    print(f"  DHAT MEMORY REPORT: {path.name}")
    print("=" * 90)
    print(f"  Max Heap (Peak est.):  {peak_bytes:>14,} bytes ({peak_bytes / (1024*1024):.2f} MB)")
    print(f"  Total Allocated:       {total_bytes:>14,} bytes ({total_bytes / (1024*1024):.2f} MB)")
    print(f"  Total Alloc Blocks:    {total_blocks:>14,}")
    print(f"  Live Bytes at Exit:    {leaked_bytes:>14,} bytes")
    print("=" * 90)

    # Sort call-sites descending by total allocated bytes
    sorted_pps = sorted(pps, key=lambda p: p.get("tb", 0), reverse=True)

    print("  TOP ALLOCATION SITES (FULL RECURSIVE CALL PATHS):")
    for i, p in enumerate(sorted_pps[:5]):
        tb = p.get("tb", 0)
        tbk = p.get("tbk", 0)
        mb = p.get("mb", 0)
        eb = p.get("eb", 0)

        print(f"\n[{i+1}] {tb:,} bytes ({tb / (1024*1024):.2f} MB) | Blocks: {tbk:,} | Peak: {mb:,} bytes | Live at exit: {eb:,} bytes")
        print("-" * 90)

        # In DHAT, fs[0] is the allocation leaf; reverse it so main() is at the top
        frame_indices = list(reversed(p.get("fs", [])))

        for depth, idx in enumerate(frame_indices):
            indent = "  " * depth
            frame_name = ftbl[idx] if idx < len(ftbl) else f"[unknown frame {idx}]"
            print(f"  {indent}↳ {frame_name.strip()}")

    print("\n" + "=" * 90)
