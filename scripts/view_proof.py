import json
import os
import re
import qcel_howmany
from qcel_howmany.qcel_howmany import Instr
from build_prover import build_prover

import sys

from view_rules import grids_levelup, instr_to_tq, show_rules_pdf

def show_proof_pdf(proof: qcel_howmany.Proof):
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
    os.system(f"typst c /tmp/rules_view.typ /tmp/rules_view.pdf")

def circuit_to_quantikz(instrs: list[Instr]) -> str:
    nqubits = max(q for instr in instrs for q in instr.qargs) + 1
    grids = [[] for i in range(nqubits)]

    for instr in instrs:
        match instr.gate.name:
            case "cx" | "cy" | "cz":
                [control, target] = instr.qargs
                grids_levelup(grids, min(control, target), max(control, target))
                grids[control].append("\\ctrl{" + str(target - control) + "}")
                grids[target].append({
                    "cx": "\\targ{}",
                    "cy": "\\gate{Y}",
                    "cz": "\\control{}",
                }[instr.gate.name])
            case "swap":
                [q1, q2] = instr.qargs
                if q1 > q2:
                    q1, q2 = q2, q1
                grids_levelup(grids, q1, q2)
                grids[q1].append("\\swap{" + str(q2 - q1) + "}")
                grids[q2].append("\\targX{{}}")
            case _:
                [q] = instr.qargs
                if instr.gate.name == 'rz':
                    param = instr.gate.params[0].replace("theta1", "theta_1").replace("theta2", "theta_2")
                    name = f"$R_z({param}$)"
                else:
                    name = instr.gate.name.upper()
                if name.endswith("DG"):
                    name = name[:-2] + r"^\dagger"
                grids[q].append(f"\\gate{{{name}}}")
    
    grids_levelup(grids, 0, nqubits - 1)
    for i in range(len(grids[0])):
        max_size = max(len(row[i]) for row in grids)
        for row in grids:
            while len(row[i]) < max_size:
                row[i] += " "
    result = "\\begin{quantikz}[column sep=0.3cm]\\\\\n"
    for row in grids:
        result +=  "&" + " & ".join(cell for cell in row) + "& \\\\\n"
    result += "\\end{quantikz}"
    return result

def show_proof_latex(proof: qcel_howmany.Proof) -> str:
    circuits = [(identity, rule) for identity, rule in proof.raw]
    rows = [r"\begin{longtable}{|c|c|}\hline Rule & Circuit \\ \hline"]
    for i, (circ, rule) in enumerate(circuits):
        text = f"{rule[0]}, {rule[1]}" if rule else "ASSUME"
        circ = circ.inner.instrs_with_swaps()
        rows.append(f"\t{i} $\\leftarrow$ {text} & {circuit_to_quantikz(circ)} \\\\\n\\hline")
    rows.append(r"\end{longtable}")
    return "\n".join(rows)

if __name__ == "__main__":
    gate_set_name = sys.argv[1]
    ngates = int(sys.argv[2])
    identity = sys.argv[3]
    circuit = []
    for gate in identity.split(";"):
        m = re.match(r"(.+)\(([\d\s,]*)\)$", gate.strip())
        name = m[1]
        qubits = [int(x) for x in m[2].split(",")]
        circuit.append(qcel_howmany.Gate(name)(*qubits))
    
    identity_circ = qcel_howmany.Circ(circuit).rotate_representative()
    prover, _ = build_prover(gate_set_name, ngates=ngates)
    if proof := prover.export_proof(identity_circ, 5, 50000):
        if "--latex" in sys.argv:
            print(show_proof_latex(proof))
        else:
            show_proof_pdf(proof)
    else:
        print("Could not prove identity.")

