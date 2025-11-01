

import quclif
from build_prover import build_prover

NGATES = {
    "logical": 8,
    "clifford": 8,
    "clifford-t": 8,
    "common-clifford-t": 6,
}

if __name__ == "__main__":

    for gate_set_name, gate_count in NGATES.items():
        for ngates in range(1, NGATES[gate_set_name] + 1):
            print(f"Building prover for gate set '{gate_set_name}' with {ngates} gates.")
            build_prover(gate_set_name, ngates=ngates)
        