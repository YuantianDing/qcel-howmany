# Python API Reference

This document describes the Python API under [`python/`](./python), specifically:

- top-level helpers in [`python/qcel_howmany/__init__.py`](./python/qcel_howmany/__init__.py),
- Rust extension types exposed via [`python/qcel_howmany/qcel_howmany.pyi`](./python/qcel_howmany/qcel_howmany.pyi).

## Modules

- `qcel_howmany`: high-level wrapper module (Qiskit interop + re-exports).
- `qcel_howmany.qcel_howmany`: low-level PyO3 extension module (core classes).

## Top-Level API (`qcel_howmany`)

#### `DEFAULT_GATES: list[str]`
Default built-in gate names:
`["H", "X", "Z", "Y", "T", "TDG", "S", "SDG", "CX", "CY", "CZ", "CS", "CSDG", "SWAP"]`

#### `Gate(instr: str | qiskit.circuit.Instruction) -> qcel_howmany.qcel_howmany.Gate | None`
Create a Rust gate object from a gate name or a Qiskit instruction:
- if `instr` is `str`, resolves via `Gate.from_name(name)`,
- otherwise builds a parameterized gate from a Qiskit instruction name, params, and matrix.

#### `Instruction(instr: qiskit._accelerate.circuit.CircuitInstruction) -> qcel_howmany.qcel_howmany.Instruction`
Convert one Qiskit `CircuitInstruction` into the high-level Rust `Instruction` type
(register-aware quantum/classical args).

#### `Circuit(circuit: qiskit.circuit.QuantumCircuit, with_regs: bool = False) -> list[Instr] | list[Instruction]`
Convert a Qiskit circuit to Rust instruction objects.

Args:
- `circuit`: Input Qiskit circuit.
- `with_regs`: If `False` (default), returns compact `Instr` with dense integer qubit ids.
  If `True`, keeps register/index labels and returns `Instruction`.

#### `to_qiskit(instrs: list[Instr] | list[Instruction]) -> qiskit.circuit.QuantumCircuit`
Convert Rust instructions (`Instruction` or `Instr`) back to a Qiskit `QuantumCircuit`.
- For `Instr`: creates a dense `QuantumCircuit(nqubits)`.
- For `Instruction`: reconstructs named quantum/classical registers.

### Top-level re-exports
The wrapper re-exports these extension classes directly:

- `ECCs`
- `ECC`
- `Instr`
- `StateVec`
- `Circ`
- `IdentityCirc`
- `IdentityProver`
- `RawECCs`
- `Evaluator`
- `Proof`

## Extension API (`qcel_howmany.qcel_howmany`)

### Data/Instruction Types

#### `Argument(regid: str, index: int)`
Named register argument (`q[0]`, `c[2]` style), e.g. `Argument("q", 0)`.
- Fields: `regid`, `index`.

#### `Gate`
Parameterized gate object.
- Properties:
  - `name` — Gate name.
  - `params` — Symbolic parameter list.
  - `params_f` — Numeric parameters evaluated with support for `pi` expressions.
  - `matrix` — Dense unitary matrix.
  - `nqargs` — Number of qubit arguments required by this gate.
- Methods:
  - `from_name(name: str) -> Gate | None` — Looks up a built-in gate by display name.
  - `adjoint() -> Gate` — Returns the adjoint (inverse) gate.
  - `__call__(*args) -> Instr | Instruction` — Binds arguments and returns either compact `Instr` (integer qargs) or high-level `Instruction` (register-style args).

#### `Instr(gate: Gate, qargs: Sequence[int])`
Compact instruction with integer qubit ids.
- Properties:
  - `gate` — Underlying gate.
  - `qargs` — Qubit arguments as dense integer indices.
- Methods:
  - `apply_permutation(perm: list[int]) -> Instr` — Applies a qubit permutation.
  - `permut(perm: list[int]) -> Instr` — Alias of `apply_permutation`.
  - `arg_mask() -> int` — Bitmask of qubits touched by this instruction.
  - `pass_mask(mask: int) -> int | None` — Updates a frontier mask for left-to-right scheduling checks.
  - `largest_qubit() -> int` — Largest qubit index touched by this instruction.
  - `position_of_qubit(qubit: int) -> int | None` — Returns position of `qubit` in argument list, if present.
  - `disjoint(other: Instr) -> bool` — Returns `true` when this and `other` touch disjoint qubits.
  - `adjoint() -> Instr` — Returns the adjoint instruction.

#### `Instruction(gate: Gate, qargs: Sequence[Argument | tuple[str, int]], cargs: Sequence[Argument | tuple[str, int]] = [])`
High-level instruction with register-aware quantum/classical args.
- Properties: `gate`, `qargs`, `cargs`.

### Circuit & Identity Types

#### `Circ(instrs: Sequence[Instr], perm: list[int] | None = None)`
Circuit with explicit trailing permutation. Creates a circuit from instructions and an optional permutation.
- Properties: `instrs`, `perm`.
- Methods:
  - `nqubits() -> int` — Number of qubits tracked by the permutation.
  - `rotate_representative() -> IdentityCirc` — Returns canonical identity representative across rotations/permutations.
  - `representative() -> Circ` — Returns canonical circuit representative.
  - `representative_with_perm() -> tuple[Circ, list[int]]` — Returns `(representative, applied_perm)` used in canonicalization.
  - `permut(perm: list[int]) -> Circ` — Applies a qubit permutation to this circuit.
  - `remove_swaps() -> Circ` — Removes explicit swap gates into the stored trailing permutation.
  - `inverse() -> Circ` — Returns inverse circuit.
  - `rotate(n: int) -> Circ` — Circularly rotates instruction order by `n`.
  - `len() -> int` — Number of instructions.
  - `is_empty() -> bool` — Returns whether the circuit is empty.
  - `instrs_with_swaps() -> list[Instr]` — Returns instructions plus swaps needed for stored permutation.

#### `IdentityCirc(circuit: Circ)`
Canonical identity circuit wrapper.
- Property: `inner`.
- Static:
  - `from_python(obj: Any) -> IdentityCirc`: Create the object from a serialized Python JSON object.
- Methods:
  - `nqubits() -> int`
  - `qargs_forward(gate_id: int) -> list[int]`
  - `qargs_backward(gate_id: int) -> list[int]`
  - `check() -> bool`
  - `hash_value() -> int`
  - `pythonize() -> Any`

#### `Proof(data: Sequence[tuple[IdentityCirc, tuple[int, int] | None]])`
Proof DAG export format. Creates a proof from raw `(identity, dependency)` entries.
- Property:
  - `raw` — Returns raw proof entries.

### ECC/Search Types

#### `ECC`
Single equivalence class.
- Methods:
  - `circuits() -> list[list[Instr]]` — Returns all circuits in this class with swap instructions materialized.

#### `ECCs`
Collection of equivalence classes for fixed qubit count.
- Property: `nqubits`.
- Static:
  - `from_postcard(filepath: str) -> ECCs` — Loads ECCs from a postcard file.
- Methods:
  - `dump_postcard(filepath: str) -> None` — Saves ECCs as postcard.
  - `dump_quartz(filepath: str) -> None` — Exports ECCs to Quartz JSON format.
  - `check() -> list[ECC]` — Returns classes that fail randomized equivalence checks.
  - `to_list() -> list[ECC]` — Returns all classes as a Python list.
  - `filter_single() -> ECCs` — Removes classes with only one circuit.
  - `to_identity_circuits() -> list[tuple[IdentityCirc, Circ, Circ]]` — Converts classes into canonical identities plus witness circuit pairs.

#### `Evaluator(nqubits: int)`
Randomized evaluator for hashing/equivalence metadata. Creates a random evaluator for `nqubits`.
- Methods:
  - `nqubits() -> int` — Returns evaluator qubit count.
  - `evaluate(instrs: Sequence[Instr]) -> tuple[StateVec, list[int], list[int], StateVec]` — Python wrapper for circuit evaluation.
  - `initial_key() -> int` — Hash key for the empty circuit under this evaluator.

#### `RawECCs(evaluator: Evaluator)`
Mutable hash-bucket map prior to simplification. Initializes the raw map with the empty circuit class.
- Static:
  - `search(evaluator, instrs, max_size) -> tuple[RawECCs, list[int]]` — Optimized search using permutation-equivalent placements per circuit.
  - `search_naive(evaluator, instrs, max_size) -> tuple[RawECCs, list[int]]` — Exhaustive baseline search without permutation-based multiplicity reduction.
  - `generate(evaluator, gates, max_size) -> tuple[RawECCs, list[int]]` — Generates ECCs from a gate set using the optimized search. Adjoint gates are added automatically if missing.
  - `generate_naive(evaluator, gates, max_size) -> tuple[RawECCs, list[int]]` — Generates ECCs from a gate set using the naive search.
- Methods:
  - `simplify() -> ECCs` — Converts raw buckets into sorted, normalized ECCs.
  - `find_equivalents(evaluator: Evaluator, instrs: Sequence[Instr]) -> ECC | None` — Python wrapper returning equivalent circuits for a candidate program.
  - `compute_next_key(evaluator: Evaluator, current_key: int, instr: Instr) -> int | None` — Computes the next reachable ECC hash after appending one instruction. Returns `None` when the resulting state was not discovered in this map.
  - `switch_evaluator(new_evaluator: Evaluator) -> RawECCs` — Re-evaluates all stored circuits under a new evaluator. Useful for cross-checking evaluator-independence of discovered classes.
  - `check_identity_subset(ecc1: RawECCs, evaluator: Evaluator) -> None` — Checks that every identity in `self` is representable in `ecc1`.

### Prover Types

#### `IdentityProver`
Rule-based identity prover built from ECCs.
- Static:
  - `build_from_eccs(eccs: ECCs) -> IdentityProver` — Python entrypoint: builds a prover from generated ECCs.
  - `from_postcard(filepath: str) -> IdentityProver` — Loads a prover from postcard serialization.
- Methods:
  - `par_apply_rules(identity, additional_size) -> list[IdentityCirc]` — Applies one round of parallel rule rewriting bounded by `additional_size`.
  - `add_identity(identity, additional_size, count_limit) -> bool` — Inserts one identity into the prover, deriving it when possible. Returns `True` when this identity must be kept as an assumption.
  - `add_identity_search(identity, additional_size, count_limit) -> tuple[set[IdentityCirc], bool]` — Internal bounded search used by `add_identity`. Returns `(visited, assumed)` where `assumed=True` means the identity could not be derived within `count_limit`.
  - `prove_identity(identity, additional_size, count_limit) -> IdentityCirc | None` — Attempts to prove one identity. Returns `None` if proved, or the identity if the search exceeded limits.
  - `prove_identity_with_visited(identity, additional_size, count_limit) -> tuple[IdentityCirc | None, set[IdentityCirc]]` — Same as `prove_identity` but also returns visited identities.
  - `export_proof(identity, additional_size, count_limit) -> Proof | None` — Exports a proof DAG when `identity` is derivable under the given limits.
  - `get_assumed() -> list[IdentityCirc]` — Returns current assumptions as a cloned list (Python helper).
  - `dump_postcard(filepath: str) -> None` — Stores a prover as postcard serialization.

### Utility Types

#### `StateVec(re: Sequence[float], im: Sequence[float])`
Complex state-vector type. Creates a state vector from real/imaginary coefficient arrays.
- Static:
  - `random(num_qubits: int) -> StateVec` — Creates a random normalized state.
  - `random_symmetric(num_qubits: int) -> StateVec` — Creates a random state tailored for symmetry-aware evaluator use.
- Methods:
  - `hash_value() -> int` — Hashes the current normalized state.
  - `nqubits() -> int` — Returns qubit count.
  - `len() -> int` — Returns vector length (`2^nqubits`).
  - `normalize() -> None` — Normalizes in place: ensures amplitude normalization (`sum(|ψᵢ|²) = 1`) and phase normalization (first non-zero element is real and positive). Degenerates to `|0…0⟩` for a near-zero vector.
  - `normalize_arg() -> None` — Canonicalizes global phase for stable equality/hash comparison.
  - `clone() -> StateVec` — Returns a deep copy.
  - `check() -> bool` — Checks unit-norm invariant.
  - `__getitem__(index: int) -> complex` — Returns one amplitude as Python `complex`.
  - `__setitem__(index: int, value: complex) -> None` — Sets one amplitude from Python `complex`.
  - `__imul__(instr: Instr) -> StateVec` — In-place application of one instruction (`state *= instr`).

#### `OrderInfo(size: int)`
Symmetry/equality-class metadata helper. Creates a single-class ordering of `size` qubits.
- Methods:
  - `n_eqclasses() -> int` — Number of current equality classes.
  - `has_eq() -> bool` — Returns true if any equality class has size > 1.
  - `first_eqclass() -> int | None` — Index of first non-trivial equality class.
  - `first_eqclass_after(idx: int) -> int | None` — Index of first non-trivial equality class at/after `idx`.
  - `as_bits() -> tuple[list[int], int]` — Returns canonical permutation and equality bitmask.

## Notes

- Canonical source of signatures and short method docs is
  [`python/qcel_howmany/qcel_howmany.pyi`](./python/qcel_howmany/qcel_howmany.pyi).
- The wrapper API in [`python/qcel_howmany/__init__.py`](./python/qcel_howmany/__init__.py)
  provides the recommended Qiskit interop entry points.
- `Gate.from_name(...)` may return `None` for unknown gate names; validate user-provided names.
