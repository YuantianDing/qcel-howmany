

import quclif
from tqdm import tqdm
from build_prover import build_prover
from prove import prove
from generate_eccs import generate_eccs

NGATES = {
    "logical": (9, 9, 9),
    "clifford": (6, 8, 6),
    "clifford-t": (6, 8, 6),
    "common-clifford-t": (5, 5, 4),
    "clifford-t1/2": (6, 7, 5),
    "clifford-rz(pi/3)": (6, 7, 5),
}

if __name__ == "__main__":
    max_size = max(v[1] for v in NGATES.values())
    for size in range(1, max_size + 1):
        for gate_set_name, (gate_count, prove_gate_count, naive_gate_count) in NGATES.items():
            if size > prove_gate_count:
                continue
            prover, _ = build_prover(gate_set_name, ngates=min(size, gate_count))
            if prover is not None:
                if gate_count < size <= prove_gate_count:
                    prove(prover, gate_set_name, ngates=size)
            if size <= naive_gate_count:
                prover, _ = build_prover(gate_set_name, ngates=min(size, gate_count), naive=True)
        