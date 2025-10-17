import json
from qiskit import quantum_info as qi
import sys
import quclif
from tqdm import tqdm
from permuta import Perm
import qiskit
from qiskit.circuit import library as qlib

from quclif import Circ, Gate, IdentityCirc, Instr, IdentitySet

gates = [qlib.HGate(), qlib.TGate(), qlib.TdgGate(), qlib.XGate(), qlib.CXGate(), qlib.SwapGate()]
gates = [Gate(g) for g in gates]
SWAP_RULE = [Gate("CX")(0, 1), Gate("CX")(1, 0), Gate("CX")(0, 1), Gate("SWAP")(0, 1)]

ADDTIONAL_RULES = [ SWAP_RULE ]

QMAP = {"Q0": 0, "Q1": 1, "Q2": 2, "Q3": 3, "Q4": 4, "Q5": 5, "Q6": 6, "Q7": 7, "Q8": 8, "Q9": 9,}

def quartz_to_circuit(data: dict, nqubits=-1) -> Circ:
    if nqubits < 0:
        nqubits = data[0][0]
    circ = data[1]
    return Circ([Gate(a[0])(*([QMAP[a] for a in a[1]])) for a in circ])

def main():
    if sys.argv[1] == "-r":
        with open(sys.argv[2]) as f:
            rules = json.load(f)
        circuits = [IdentityCirc.from_python(rule) for rule in rules]
        circuits.sort(key=lambda c: (len(c), c))
        print('#import "@preview/quill:0.7.2" as quill: tequila as tq')
        for i, c in enumerate(circuits):
            # assert qi.Operator(c.to_qiskit()).equiv(qi.Operator(qiskit.QuantumCircuit(c.nqubits())))
            lst = ", ".join(f"tq.{instr}" for instr in c.inner.instrs_with_swaps())
            print(f'{i}. #quill.quantum-circuit(..tq.build({lst}))\n')

    else:
        print("Parsing JSON.")
        eccset = []
        for path in sys.argv[1:]:
            with open(path) as f:
                eccset.extend(json.load(f)[1].values())

        print("Loading Identities.")
        identities = []
        for ecc in tqdm(eccset):
            nqubits = max(c[0][0] for c in ecc)
            ecc = [quartz_to_circuit(c, nqubits) for c in ecc]
            initial = ecc[0].inverse()
            for c in ecc[1:]:
                id = (initial + c).rotate_representative()
                identities.append(id)

        for rule in ADDTIONAL_RULES:
            identities.append(Circ(rule).rotate_representative())

        print("Sorting Identities.")
        identities.sort(key=lambda c: (len(c), c))
        
        print("Finding Minimal Identies.")
        idset = IdentitySet()
        for id in tqdm(identities):
            if some := idset.add_identity(id, max_step=10):
                tqdm.write(f"{some} {some.nqubits()}")
        print(idset.count_rules())
        idset = idset.as_list()
        print(len(idset))
        with open("rules.json", "w") as f:
            json.dump([id.pythonize() for id in idset], f)

if __name__ == "__main__":
    main()
