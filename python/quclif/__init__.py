from numpy._core.numeric import allclose
import qiskit
import qiskit.circuit
from qiskit.quantum_info import Operator
import quclif.quclif as inner
from fractions import Fraction
import math

def format_number(num: float):
    if allclose(num, int(num)):
        return f"{int(num)}"

    n, d = num.as_integer_ratio()
    if d <= 10000:
        return f"{n}/{d}"
    n, d = (num / math.pi).as_integer_ratio()
    if d <= 10000:
        if n == 1:
            return f"pi/{d}"
        elif n == -1:
            return f"-pi/{d}"
        else:
            return f"{n}pi/{d}"
    return f"{num:.6g}"


def Gate(instr: qiskit.circuit.Instruction):
    return inner.Gate(instr.name, [format_number(p) for p in instr.params], Operator(instr).reverse_qargs().to_matrix())

def Instruction(instr: qiskit._accelerate.circuit.CircuitInstruction):
    return inner.Instruction(Gate(instr.operation), [(qbit._register.name, qbit._index) for qbit in instr.qubits], [(cbit._register.name, cbit._index) for cbit in instr.clbits])

def Circuit(circuit: qiskit.circuit.QuantumCircuit) -> list[inner.Instruction]:
    return [Instruction(instr) for instr in circuit.data]

def to_qiskit(instrs: list[inner.Instruction]):
    from collections import defaultdict

    qregs = defaultdict(int)
    for instr in instrs:
        for qbit in instr.qargs:
            qregs[qbit.regid] = max(qregs[qbit.regid], qbit.index + 1) 
    qregs = {regid: qiskit.circuit.QuantumRegister(size, regid) for regid, size in qregs.items()}

    cregs = defaultdict(int)
    for instr in instrs:
        for cbit in instr.cargs:
            cregs[cbit.regid] = max(cregs[cbit.regid], cbit.index + 1)
    cregs = {regid: qiskit.circuit.ClassicalRegister(size, regid) for regid, size in cregs.items()}

    circuit = qiskit.circuit.QuantumCircuit(*qregs.values(), *cregs.values())

    for instr in instrs:
        getattr(circuit, instr.gate.name)(
            *instr.gate.params_f,
            *[qiskit.circuit.Qubit(qregs[qbit.regid], qbit.index) for qbit in instr.qargs],
            *[qiskit.circuit.Clbit(cregs[cbit.regid], cbit.index) for cbit in instr.cargs],
        )

    return circuit

CircitECCs = inner.CircuitECCs
StateVec = inner.StateVec