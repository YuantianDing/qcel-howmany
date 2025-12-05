
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
    if instr.gate.name == "rz":
        param = instr.gate.params[0].replace("theta1", "theta_1").replace("theta2", "theta_2")
        return f"tq.rz(${param}$, {instr.qargs[0]})"
    else:
        return f"tq.{instr}"

def instrs_to_qiskit(instrs: list[Instr]) -> qiskit.QuantumCircuit:
    nqubits = max(q for instr in instrs for q in instr.qargs) + 1
    circuit = qiskit.QuantumCircuit(nqubits)
    for instr in instrs:
        getattr(circuit, instr.gate.name)(*instr.qargs)
        
    return circuit

def show_rules_pdf(file: str):
    with open(file) as f:
        rules = json.load(f)
        if 'rules' in rules:
            rules = rules['rules']
        circuits = [IdentityCirc.from_python(rule) for rule in rules]
        circuits.sort(key=lambda c: (len(c), c))
        print("\\and \n".join([circuit_to_quantikz(c.inner.instrs_with_swaps()) for c in circuits]))
    with open("/tmp/rules_view.typ", "w") as f:
        f.write('#import "@preview/quill:0.7.2" as quill: tequila as tq\n')
        for i, c in enumerate(c for c in circuits if len(c) > 0):
            # assert qi.Operator(c.to_qiskit()).equiv(qi.Operator(qiskit.QuantumCircuit(c.nqubits())))
            # print(c.inner.instrs_with_swaps())
            lst = ", ".join(instr_to_tq(instr) for instr in c.inner.instrs_with_swaps())
            f.write(f'{i}. #quill.quantum-circuit(..tq.build({lst}))\n\n')
    os.system(f"typst c /tmp/rules_view.typ {file.replace('json', 'pdf')} && code {file.replace('json', 'pdf')}")
    

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
    result = "\\begin{tikzpicture}\\node[scale=0.8]{\\begin{quantikz}[column sep=0.3cm]\n"
    for row in grids:
        result +=  "&" + " & ".join(cell for cell in row) + "& \\\\\n"
    result += "\\end{quantikz}};\\end{tikzpicture}"
    return result
            
def grids_levelup(grids: list[list[str]], f: int, t: int):
    maxlen = max(len(row) for row in grids[f:t+1])
    for row in grids[f:t+1]:
        while len(row) < maxlen:
            row.append("")



if __name__ == "__main__":
    show_rules_pdf(sys.argv[1])