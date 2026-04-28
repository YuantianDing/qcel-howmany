
# Artifact for CAV 2026

This repository is the artifact for the CAV 2026 paper
*"How Many Quantum Circuit Identities Are Needed to Generate All Others?"*
The paper PDF is included in this repo: [`cav26.pdf`](./cav26.pdf).

Links: [Rust API](https://yuantianding.github.io/qcel-howmany/) [Python API](https://github.com/YuantianDing/qcel-howmany/blob/main/PYTHON-API.md)

## Overview

The artifact is a Rust core (with Python bindings via PyO3) and a set of Python
driver scripts that reproduce every experimental table and figure in the paper.
It covers six gate sets — `logical`, `clifford`, `clifford-t`,
`common-clifford-t`, `clifford-t1/2`, and `clifford-rz(pi/3)` — and includes:

- ECC synthesis and pruning (Section 5.1),
- a comparison against a naive baseline (Section 5.2.1),
- proving identities from existing Quartz ECC sets (Section 5.2.2),
- a fixed-point precision study (Section 5.2.3), and
- an exportable proof viewer (Section 5.2.4).

For evaluation, we recommend a machine with at least 16-core CPU and 32 GB of RAM. A
full sweep may take one to two days; you can stop at any point and inspect the
partial results, or run a single configuration for a quick check.

## Setup

### Option 1 — Docker (recommended)

Pull the prebuilt image:

```bash
docker pull yuantianding/qcel-howmany
docker run -it --rm yuantianding/qcel-howmany
```

Or build it locally from this repository (need Internet access to build the image):

```bash
docker build -t qcel-howmany .
docker run -it --rm qcel-howmany
```

The container drops you into a shell at `/workspace/qcel_howmany` with the
package already installed and `typst` on the `PATH`.

Note that to trim down the image size, the docker container does not include the building toolchain to recompile the Rust code into a Python module. We use Debian's `slim-trixie` image as the base image.

### Option 2 — Local build

Requirements:

- Rust toolchain (stable)
- Python ≥ 3.8
- [maturin](https://www.maturin.rs/) (builds the Rust ↔ Python bindings)
- [uv](https://docs.astral.sh/uv/) — optional, used for environment management
- [typst](https://typst.app/) — optional, only needed for proof/rule visualization

Build and install the wheel into a fresh virtualenv:

```bash
uv sync                                  # create .venv and install Python deps
uv run maturin develop --release         # compile the Rust core into the venv
```

Or, without `uv`:

```bash
python3 -m venv .venv && source .venv/bin/activate
pip install maturin pandas tqdm tables pillow pylatexenc qiskit
maturin develop --release
```

All commands below assume you run them from the repository root with the venv
active (or inside the Docker container).

## Repository Layout

```text
.
├── src/                 # Rust core library and PyO3 bindings
│   ├── lib.rs           # crate entry, module wiring, Python module exports
│   ├── search/          # ECC generation and search
│   ├── identity/        # identity representation, prover, proof export
│   ├── circ/            # gates, instructions, circuit data structures
│   ├── state/           # state-vector simulation and order information
│   ├── groups/          # group-theoretic helpers used by the prover
│   ├── utils/           # shared utilities
│   ├── qreal_f64.rs     # f64 backend for fixed-/floating-point amplitudes
│   ├── qreal_f128.rs    # f128 backend (enabled with the `f128` feature)
│   └── bin/             # standalone Rust binaries (synthesis, stub gen, …)
├── python/qcel_howmany/ # Python package and generated stubs (.pyi)
├── scripts/             # driver scripts for synthesis / proving / tables
│   ├── run_all.py           # full sweep across gate sets and sizes
│   ├── run_once.py          # one (gate set, n) configuration
│   ├── print_table.py       # render summary tables (text or LaTeX)
│   ├── prove_quartz.py      # prove identities from a Quartz ECC set
│   ├── run_floatpoints.py   # fixed-point precision experiment
│   ├── view_proof.py        # render a proof of a specific identity
│   ├── view_rules.py        # render the rule set for a gate set
│   ├── build_prover.py      # library: build a prover for one configuration
│   ├── generate_eccs.py     # library: synthesize ECCs
│   ├── prove.py             # library: prove identities
│   └── gate_set.py          # gate-set definitions used by all scripts
├── quartz/              # Quartz ECC sets used in Section 5.2.2
├── Cargo.toml           # Rust crate manifest
├── pyproject.toml       # Python build metadata (maturin backend)
└── Dockerfile
```

## Reproducing the Experiments

Before going into the commands, we first the gate sets configuration used in our `scripts/`.

## Gate Set Configuration

Gate sets are defined in [`scripts/gate_set.py`](./scripts/gate_set.py):

```python
GATE_SETS = {
    "logical":            ["X", "CX"],
    "clifford":           ["H", "S", "SDG", "X", "CX"],
    "clifford-t":         ["H", "T", "TDG", "X", "CX"],
    "common-clifford-t":  ["H", "S", "T", "TDG", "X", "CX", "CY", "CZ", "Y", "Z", "S", "SDG"],
    "clifford-t1/2":      ["H", "T1/2", "TDG1/2", "S", "SDG", "X", "CX"],
    "clifford-rz(pi/3)":  ["H", "S", "SDG", "X", "CX", "RZ(pi/3)", "RZ(-pi/3)"],
}
```

The artifact uses the name `logical` for the reversible classical gate set `{X, CX}`; the paper text refers to this set as `classic`.

Experiments configurations are shown in [`scripts/run_all.py`](./scripts/run_all.py) in the `NGATES` dictionary:

```python
NGATES = {
    # number of gates to (build prover, prove, run naive method)
    "logical": (9, 9, 9),
    "clifford": (6, 8, 6),
    "clifford-t": (6, 8, 6),
    # "common-clifford-t": (5, 5, 4),
    # "clifford-t1/2": (6, 7, 5),
    # "clifford-rz(pi/3)": (6, 7, 5),
}
```

The first number is the highest number of gates for which we build a "prover", i.e. `rules` table defined in Algorithm 3 of the paper, which consumes a lot of memory and time for larger gate sets, and run full version of Algorithm 3 of the paper. The second number is the highest number of gates for which we run the full synthesis, but use the previous prover to prove that no additional rules are needed. The third number is the highest number of gates for which we run the naive method, which is much slower and memory intensive than the optimized method, so we only run it for smaller gate counts.

In this artifact, by default, we only run 3 gate sets (`logical`, `clifford`, and `clifford-t`) for simplicity and to reduce the total runtime. The other three gate sets (`common-clifford-t`, `clifford-t1/2`, and `clifford-rz(pi/3)`) are commented out in the above dictionaries; you can uncomment them to run them as well.

### 5.1 — Synthesis and Pruning

Run the full sweep:

```bash
python3 scripts/run_all.py
```

This populates `.cache/` with intermediate artifacts. Typical filenames:

- `.cache/eccset-<gate_set>-<ngates>-5.eccs` (+ `.json`)
- `.cache/prover-<gate_set>-<ngates>-5.prover` (+ `.json`)
- `.cache/prove-<gate_set>-<ngates>-5.json` (proving-only configurations)

Render the summary tables:

```bash
python3 scripts/print_table.py            # text
python3 scripts/print_table.py --latex    # LaTeX
```

`scripts/print_table.py` can be run before `scripts/run_all.py` finishes; unfinished configurations are shown as `w`.

Cells marked `w` are configurations that have not finished yet.

To run a single configuration:

```bash
python3 scripts/run_once.py <gate_set_name> <ngates>
```

The set of (gate set, n) targets is configured in the `NGATES` dictionary at
the top of [`scripts/run_all.py`](./scripts/run_all.py); edit it to run a
subset.

Note that due to a limitation of PyO3, it is impossible to `KeyboardInterrupt` a long-running Rust function from Python. If you need to stop a long-running configuration, you can `kill` the process from another terminal; the intermediate results up to that point will still be cached in `.cache/`.

### 5.2.1 — Comparison with a Naive Baseline

`scripts/run_all.py` will automatically run the naive baselines. To view the result of naive method, add `--naive` to these commands:

```bash
python3 scripts/print_table.py --naive
python3 scripts/run_once.py <gate_set_name> <ngates> --naive
```

### 5.2.2 — Proving Existing Quantum Optimizations (Quartz)

Quartz ECC sets are in [`quartz/`](./quartz). Usage:

```bash
python3 scripts/prove_quartz.py <gate_set_name> <ngates> <quartz_ecc>.json
```

The exact commands used in the paper:

```bash
python3 scripts/prove_quartz.py logical            6 quartz/classic-5complete_ECC_set.json
python3 scripts/prove_quartz.py clifford           6 quartz/clifford-5complete_ECC_set.json
python3 scripts/prove_quartz.py clifford-t         6 quartz/clifford-t-5complete_ECC_set.json
python3 scripts/prove_quartz.py common-clifford-t  5 quartz/common-clifford-t-5complete_ECC_set.json
python3 scripts/prove_quartz.py clifford-t1/2      5 quartz/clifford-t1-2complete_ECC_set.json
python3 scripts/prove_quartz.py "clifford-rz(pi/3)" 5 quartz/clifford-rz-pi-3pruning_unverified.json
```

To generate these ECC sets using Quartz, we provide another Docker image `yuantianding/quartz-gen-ecc` at `docker.io`. Running this image directly with `docker run` will automatically generate the ECC set files mentioned above using Quartz, under `/quartz/eccset` directory. Using `docker cp` can directly extract these files from the Docker container. It's worthy to note that we edited Quartz to add additional gates for comparison with our method. Our version of Quartz is availble at [https://github.com/YuantianDing/quartz](https://github.com/YuantianDing/quartz).

### 5.2.3 — Impact of Floating-Point Precision

```bash
python3 scripts/run_floatpoints.py
```

This invokes `cargo run --release --bin run_synthesis_fp` for several
precision levels (with and without the `f128` Cargo feature), caches the
results in `.cache/fixed_point_results.h5`, and prints a LaTeX table. The compiling and running of the Rust binary may take a while, especially for the `f128` feature.

### 5.2.4 — Exportable Proofs

Render a proof of a concrete identity, either as a Typst-generated PDF or as a
LaTeX `quantikz` snippet:

```bash
python3 scripts/view_proof.py <gate_set_name> <ngates> "<identity>"
python3 scripts/view_proof.py <gate_set_name> <ngates> "<identity>" --latex
```

Example:

```bash
python3 scripts/view_proof.py clifford 6 \
    "cx(0, 1); cx(2, 0); cx(1,2); cx(0, 1); cx(2, 0); cx(1, 2); cx(0, 1); swap(0, 2); swap(1, 2)"
```

The Typst PDF is written to `proof_view.pdf` of the current working directory. The `--latex` form prints
the snippet to stdout; the PDF form requires `typst` command on the `PATH`.

## Notes

- **Cache.** Intermediate results are stored in `.cache/`: Rust structs use
  [`postcard`](https://crates.io/crates/postcard) serialization, tabular data
  uses HDF5 (`.h5`). ECC sets and proofs are also exported as JSON for
  inspection. Delete `.cache/` to rerun from scratch.
- **Python stubs.** `python/qcel_howmany/qcel_howmany.pyi` is generated from
  Rust doc comments via the `stub_gen` binary in [`src/bin/stub_gen.rs`](./src/bin/stub_gen.rs).
- **Proof / rule visualization.** [`scripts/view_proof.py`](./scripts/view_proof.py)
  and [`scripts/view_rules.py`](./scripts/view_rules.py) shell out to `typst`
  to produce PDFs; install `typst` and ensure it is on the `PATH` to use them (It is available in the given Docker container).
