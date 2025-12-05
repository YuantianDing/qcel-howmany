import json
import os
import re
import quclif
from build_prover import build_prover

import sys

from view_rules import circuit_to_quantikz, instr_to_tq, show_rules_pdf

def show_proof_pdf(proof: quclif.Proof):
    circuits = [(identity, rule) for identity, rule in proof.raw]
    with open("/tmp/rules_view.typ", "w") as f:
        f.write('#import "@preview/quill:0.7.2" as quill: tequila as tq\n#set align(center)\n#table(columns: 2, align: horizon, [*Rule*], [*Circuit*], \n')
        for i, (circ, rule) in enumerate(circuits):
            # assert qi.Operator(c.to_qiskit()).equiv(qi.Operator(qiskit.QuantumCircuit(c.nqubits())))
            # print(c.inner.instrs_with_swaps())
            text = f"{rule[0]}, {rule[1]}" if rule else "ASSUME"
            lst = ", ".join(instr_to_tq(instr) for instr in circ.inner.instrs_with_swaps())
            f.write(f'\t[{i} $<-$ {text}], [#quill.quantum-circuit(..tq.build({lst}))],\n')
        f.write(f')\n')
    os.system(f"typst c /tmp/rules_view.typ /tmp/rules_view.pdf && code /tmp/rules_view.pdf")
    

if __name__ == "__main__":
    gate_set_name = sys.argv[1]
    ngates = int(sys.argv[2])
    identity = sys.argv[3]
    circuit = []
    for gate in identity.split(";"):
        m = re.match(r"(.+)\(([\d\s,]*)\)$", gate.strip())
        name = m[1]
        qubits = [int(x) for x in m[2].split(",")]
        circuit.append(quclif.Gate(name)(*qubits))
    
    identity_circ = quclif.Circ(circuit).rotate_representative()
    prover, _ = build_prover(gate_set_name, ngates=ngates)
    if proof := prover.export_proof(identity_circ, 5, 50000):
        show_proof_pdf(proof)
    else:
        print("Could not prove identity.")

