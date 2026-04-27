
import json
import sys

from qcel_howmany import Circ, Gate
from tqdm import tqdm
from build_prover import build_prover

QMAP = {"Q0": 0, "Q1": 1, "Q2": 2, "Q3": 3, "Q4": 4, "Q5": 5, "Q6": 6, "Q7": 7, "Q8": 8, "Q9": 9,}

def quartz_to_circuit(data: dict, nqubits=-1) -> Circ:
    if nqubits < 0:
        nqubits = data[0][0]
    circ = data[1]

    result = []
    for a in circ:
        if a[0] == "thalf":
            gate = Gate("t1/2")
        elif a[0] == "thalf_dg":
            gate = Gate("tdg1/2")
        elif a[0] == "phase_frac_pi_3":
            gate = Gate("rz(pi/3)")
        elif a[0] == "phase_frac_pi_3_dg":
            gate = Gate("rz(-pi/3)")
        else:
            gate = Gate(a[0])
        if gate is None:
            print(a[0])
        result.append(gate(*([QMAP[a] for a in a[1]])))
    return Circ(result)

if __name__ == "__main__":
    gate_set = sys.argv[1]
    ngates = int(sys.argv[2])
    quartz_file = sys.argv[3]
   
    
    print("Parsing Quartz file...")
    with open(quartz_file) as f:
        eccset = [[quartz_to_circuit(c) for c in ecc] for ecc in json.load(f)[1].values()]
        n_ecc = len(eccset)
        n_circuits = sum(len(ecc) for ecc in eccset)
        print(f"Loaded {n_ecc} ECCs with a total of {n_circuits} circuits.")
    print("Loading identities...")
    identities = []
    for i, ecc in enumerate(tqdm(eccset)):
        initial = ecc[0].inverse()
        for c in ecc[1:]:
            try:
                id = (initial + c).rotate_representative()
                identities.append(id)
            except BaseException as e:
                print(f"Error processing ECC {i}, circuit {c} == {ecc[0]}")

    prover, _ = build_prover(gate_set, ngates=ngates)
    
    print("Sorting Identities.")
    identities.sort(key=lambda c: (len(c), c))

    print("Proving identities...")
    for id in tqdm(identities):
        if a := prover.prove_identity(id, 2, 50000):
            print("Prover failed at ", a)
            break
    else:
        print("All identities proved!")


    

    
    

