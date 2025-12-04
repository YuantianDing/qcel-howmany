

import json
import os
import time
import quclif
from tqdm import tqdm

from generate_eccs import generate_eccs


def prove(prover: quclif.IdentityProver, gate_set: str, ngates: int, nqubits: int = 5, naive=False) -> dict:
    name = f".cache/prove-{gate_set.replace("/", "::")}-{ngates}-{nqubits}{'-naive' if naive else ''}.json"
    if os.path.exists(name):
        with open(name) as f:
            result = json.load(f)
        return result

    eccs, _ = generate_eccs(gate_set, ngates, nqubits, naive=naive)

    start = time.time_ns()
    rules = []
    print(f"Proving identities... (gate set '{gate_set}' with {ngates} gates.)")
    for idcirc in tqdm(eccs.to_identity_circuits()):
        if idcirc := prover.prove_identity(idcirc, 2, 50000):
            print(idcirc)
            rules.append(idcirc)
    
    result = {
        'time': time.time_ns() - start,
        'rules': [a.pythonize() for a in prover.get_assumed() + rules],
    }

    with open(name, "w") as f:
        json.dump(result, f)
    return result