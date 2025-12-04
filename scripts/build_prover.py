import json
import os
import time
from qiskit import quantum_info as qi
import sys
import quclif
from tqdm import tqdm
from permuta import Perm
import qiskit
from qiskit.circuit import library as qlib

from generate_eccs import generate_eccs

def build_prover(gate_set: str, ngates: int, nqubits: int = 5) -> tuple[quclif.IdentityProver, dict]:
    name = f".cache/prover-{gate_set.replace("/", "::")}-{ngates}-{nqubits}.prover"
    if os.path.exists(name) and os.path.exists(name + ".json"):
        print(f"Loading prover for gate set '{gate_set}' with {ngates} gates.")
        prover = quclif.IdentityProver.from_postcard(name)
        with open(name + ".json") as f:
            metadata = json.load(f)
        return prover, metadata
    
    print(f"Building prover for gate set '{gate_set}' with {ngates} gates.")
    eccs, _ = generate_eccs(gate_set, ngates, nqubits)
    
    start = time.time_ns()
    prover = quclif.IdentityProver.build_from_eccs(eccs)
    metadata = {
        'time': time.time_ns() - start,
        'rules': [a.pythonize() for a in prover.get_assumed()],
    }

    prover.dump_postcard(name)
    with open(name + ".json", "w") as f:
        json.dump(metadata, f)
    return prover, metadata

# gates = [qlib.HGate(), qlib.TGate(), qlib.TdgGate(), qlib.XGate(), qlib.CXGate(), qlib.SwapGate()]
# gates = [Gate(g) for g in gates]
# SWAP_RULE = [Gate("CX")(0, 1), Gate("CX")(1, 0), Gate("CX")(0, 1), Gate("SWAP")(0, 1)]

# ADDTIONAL_RULES = [ SWAP_RULE ]

# QMAP = {"Q0": 0, "Q1": 1, "Q2": 2, "Q3": 3, "Q4": 4, "Q5": 5, "Q6": 6, "Q7": 7, "Q8": 8, "Q9": 9,}

# def quartz_to_circuit(data: dict, nqubits=-1) -> Circ:
#     if nqubits < 0:
#         nqubits = data[0][0]
#     circ = data[1]
#     return Circ([Gate(a[0])(*([QMAP[a] for a in a[1]])) for a in circ])

# def main():
#     if sys.argv[1] == "-r":
#         pass

#     elif sys.argv[1] == "--subset":
#         with open(sys.argv[2]) as f:
#             rules1 = json.load(f)
#         with open(sys.argv[3]) as f:
#             rules2 = json.load(f)

#         circuits1 = [IdentityCirc.from_python(rule) for rule in rules1]
#         circuits2 = [IdentityCirc.from_python(rule) for rule in rules2]

#         idset = IdentitySet(max(len(c) for c in circuits1 + circuits2))
#         for c in circuits2:
#             idset.add_identity(c, max_step=3)
        

#         for c in circuits1:
#             if circ := idset.add_identity(c, max_step=50):
#                 print(circ)
#     else:
#         print("Parsing JSON.")
#         eccset = []
#         for path in sys.argv[1:]:
#             with open(path) as f:
#                 eccset.extend(json.load(f)[1].values())

#         print("Loading Identities.")
#         identities = []
#         identity_max_size = 0
#         for ecc in tqdm(eccset):
#             nqubits = max(c[0][0] for c in ecc)
#             ecc = [quartz_to_circuit(c, nqubits) for c in ecc]
#             initial = ecc[0].inverse()
#             for c in ecc[1:]:
#                 id = (initial + c).rotate_representative()
#                 identities.append(id)
#                 identity_max_size = max(identity_max_size, len(id))

#         for rule in ADDTIONAL_RULES:
#             identities.append(Circ(rule).rotate_representative())

#         print("Sorting Identities.")
#         identities.sort(key=lambda c: (len(c), c))
        
#         print("Finding Minimal Identies.")
#         idset = IdentitySet(identity_max_size)
#         for id in tqdm(identities):
#             if some := idset.add_identity(id, max_step=10):
#                 tqdm.write(f"{some} {some.nqubits()}")
#         print(idset.count_rules())
#         idset = idset.as_list()
#         print(len(idset))
#         with open("rules.json", "w") as f:
#             json.dump([id.pythonize() for id in idset], f)

# if __name__ == "__main__":
#     main()
