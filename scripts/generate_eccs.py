
import json
import os
import time
import qiskit
import qiskit.qasm2
from qiskit.circuit import library as qlib
import qiskit.quantum_info as qi
import quclif

from gate_set import GATE_SETS

def generate_eccs(gate_set: str, ngates: int, nqubits: int = 5, naive=False) -> tuple[quclif.ECCs, dict]:
    name = f".cache/eccset-{gate_set.replace("/", "::")}-{ngates}-{nqubits}{'-naive' if naive else ''}.eccs"
    if os.path.exists(name) and os.path.exists(name + ".json"):
        ecc_set = quclif.ECCs.from_postcard(name)
        with open(name + ".json") as f:
            metadata = json.load(f)
        return ecc_set, metadata
    
    start = time.time_ns()
    evaluator = quclif.Evaluator(nqubits=nqubits)
    if naive:
        (ecc_set, counters) = quclif.RawECCs.generate_naive(
            evaluator,
            gates=[quclif.Gate(g.lower()) for g in GATE_SETS[gate_set]],
            max_size=ngates,
        )
    else:
        (ecc_set, counters) = quclif.RawECCs.generate(
            evaluator,
            gates=[quclif.Gate(g.lower()) for g in GATE_SETS[gate_set]],
            max_size=ngates,
        )
    ecc_set = ecc_set.simplify()
    assert len(ecc_set.check()) == 0
    
    metadata = {
        'time': time.time_ns() - start,
        'counters': counters,
        'eccs': len(ecc_set),
        'identities': len(ecc_set.to_identity_circuits()),
    }
    
    os.makedirs(".cache", exist_ok=True)
    ecc_set.dump_postcard(name)
    with open(name + ".json", "w") as f:
        json.dump(metadata, f)
    
    return ecc_set, metadata

# if __name__ == "__main__":
#     for nqubits in range(2, 5):
#         for ngates in range(4, 7):
#             eccs = generate_eccs(nqubits, ngates, gates)
#             print(f"Generated ECCs for nqubits={nqubits}, ngates={ngates}: {len(eccs)} identities.")