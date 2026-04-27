
# Artifact for CAV 2026

This repository contains the artifact for:
"How Many Quantum Circuit Identities Are Needed to Generate All Others?"

The paper PDF is in this repo: [`cav26.pdf`](./cav26.pdf).

## Overview

The project has two main parts:

1. A Rust core (`src/`) for ECC generation, identity construction, and proving.
2. Python orchestration scripts (`scripts/`) for reproducing the experiments and tables in the paper.

Generated intermediate results are cached under `.cache/`.

## File Organization

```text
.
├── src/                         # Rust core library and Python bindings (PyO3)
│   ├── search/                  # ECC generation/search algorithms
│   ├── identity/                # Identity representation, prover, proof export
│   ├── circ/                    # Gates, instructions, and circuit data structures
│   └── state/                   # State-vector simulation and order information
├── python/qcel_howmany/         # Python package and generated stubs
├── scripts/                     # Repro scripts for synthesis/proving/tables
├── quartz/                      # Quartz ECC sets used in comparison/proving
├── circuit/                     # Benchmark/input circuit collections
├── Cargo.toml                   # Rust dependencies and crate settings
└── pyproject.toml               # Python package/build settings
```

## Setup

Prerequisites:

- Rust toolchain (for compiling `qcel_howmany`)
- Maturin (for building the Python extension module)
- Python + `uv`
- (Optional) `typst` if you want PDF visualization from `scripts/view_rules.py` / `scripts/view_proof.py`

Recommended setup:

```bash
uv sync --group dev
uv run maturin develop --release
```

This installs Python dependencies and builds the Python extension module from Rust.

## Standardized Gate Set Names

Defined in [`scripts/gate_set.py`](./scripts/gate_set.py):

```python
GATE_SETS = {
    "logical": ["X", "CX"],
    "clifford": ["H", "S", "SDG", "X", "CX"],
    "clifford-t": ["H", "T", "TDG", "X", "CX"],
    "common-clifford-t": ["H", "S", "T", "TDG", "X", "CX", "CY", "CZ", "Y", "Z", "S", "SDG"],
    "clifford-t1/2": ["H", "T1/2", "TDG1/2", "S", "SDG", "X", "CX"],
    "clifford-rz(pi/3)": ["H", "S", "SDG", "X", "CX", "RZ(pi/3)", "RZ(-pi/3)"],
}
```

Compared with the paper text, this artifact uses `logical` (instead of `classic`) for the reversible classical gate set `["X", "CX"]`.

## Reproducing the Experimental Sections

### 5.1 Synthesis and Pruning

Run the full experiment sweep:

```bash
uv run scripts/run_all.py
```

This generates ECC sets and prover outputs in `.cache/`.
Typical files include:

- `.cache/eccset-<gate_set>-<ngates>-5.eccs` and `.json`
- `.cache/prover-<gate_set>-<ngates>-5.prover` and `.json`
- `.cache/prove-<gate_set>-<ngates>-5.json` (for larger proving-only ranges)

Print summary tables:

```bash
uv run scripts/print_table.py
uv run scripts/print_table.py --latex
```

Unfinished entries are shown as `w`.

Run one configuration only:

```bash
uv run scripts/run_once.py <gate_set_name> <ngates>
```

### 5.2 Correctness

#### 5.2.1 Comparison with Naive Implementation

Use the `--naive` flag:

```bash
uv run scripts/print_table.py --naive
uv run scripts/run_once.py <gate_set_name> <ngates> --naive
```

#### 5.2.2 Proving Existing Quantum Optimizations (Quartz ECC Sets)

Quartz ECC sets are provided in [`quartz/`](./quartz).
Use:

```bash
uv run scripts/prove_quartz.py <gate_set_name> <ngates> <quartz_ecc>.json
```

Commands used in the paper:

```bash
uv run scripts/prove_quartz.py logical 6 quartz/classic-5complete_ECC_set.json
uv run scripts/prove_quartz.py clifford 6 quartz/clifford-5complete_ECC_set.json
uv run scripts/prove_quartz.py clifford-t 6 quartz/clifford-t-5complete_ECC_set.json
uv run scripts/prove_quartz.py common-clifford-t 5 quartz/common-clifford-t-5complete_ECC_set.json
uv run scripts/prove_quartz.py clifford-t1/2 5 quartz/clifford-t1-2complete_ECC_set.json
uv run scripts/prove_quartz.py clifford-rz(pi/3) 5 quartz/clifford-rz-pi-3pruning_unverified.json
```

#### 5.2.3 Impact of Floating-Point Precision

```bash
uv run scripts/run_floatpoints.py
```

This writes/reads `.cache/fixed_point_results.h5` and prints a LaTeX table.

#### 5.2.4 Exportable Proofs

Generate a visual or LaTeX proof for a concrete identity:

```bash
uv run scripts/view_proof.py <gate_set_name> <ngates> "<identity>"
uv run scripts/view_proof.py <gate_set_name> <ngates> "<identity>" --latex
```

Example:

```bash
uv run scripts/view_proof.py clifford 6 "cx(0, 1); cx(2, 0); cx(1,2); cx(0, 1); cx(2, 0); cx(1, 2); cx(0, 1); swap(0, 2); swap(1, 2)"
```

## Other Notes

- Documentation can be generated from Rust doc comments via `cargo doc` and viewed in `target/doc/`. Python API stubs are generated and documented in `python/qcel_howmany/qcel_howmany.pyi` from Rust doc/comments via `src/bin/stub_gen.rs`.
- Delete `.cache/` to rerun experiments from scratch.
- Intermediate results are stored in `.cache/` using `postcard` serialization for Rust structs and `h5` for tabular data. ECC sets and proofs are also exported in JSON format for inspection.
- `scripts/run_all.py` can be configured to run a subset of experiments by editing the `NGATES` dictionary.
- The `scripts/view_rules.py` and `scripts/view_proof.py` scripts use `typst` to generate PDF visualizations of identities and proofs. Install `typst` and ensure it's in your PATH to use this feature.


