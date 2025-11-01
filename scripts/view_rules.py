
import json
import os
import sys
from qiskit import quantum_info as qi
import qiskit
from quclif import IdentityCirc, Instr
from gate_set import GATE_SETS

def instr_to_tq(instr: Instr) -> str:
    if instr.gate.name == "cy":
        return f"tq.ca({instr.qargs[0]}, {instr.qargs[1]}, \"Y\")"
    else:
        return f"tq.{instr}"
    

def show_rules_pdf(file: str):
    with open(file) as f:
        rules = json.load(f)
        if 'rules' in rules:
            rules = rules['rules']
        circuits = [IdentityCirc.from_python(rule) for rule in rules]
        circuits.sort(key=lambda c: (len(c), c))
    with open("/tmp/rules_view.typ", "w") as f:
        f.write('#import "@preview/quill:0.7.2" as quill: tequila as tq\n')
        for i, c in enumerate(c for c in circuits if len(c) > 0):
            # assert qi.Operator(c.to_qiskit()).equiv(qi.Operator(qiskit.QuantumCircuit(c.nqubits())))
            # print(c.inner.instrs_with_swaps())
            lst = ", ".join(instr_to_tq(instr) for instr in c.inner.instrs_with_swaps())
            f.write(f'{i}. #quill.quantum-circuit(..tq.build({lst}))\n\n')
    os.system(f"typst c /tmp/rules_view.typ {file.replace('json', 'pdf')} && code {file.replace('json', 'pdf')}")
    


if __name__ == "__main__":
    show_rules_pdf(sys.argv[1])