
import qiskit
import qiskit.qasm2
from qiskit.circuit import library as qlib
import qiskit.quantum_info as qi
import quclif

gates = [qlib.HGate(), qlib.TGate(), qlib.TdgGate(), qlib.XGate(), qlib.CXGate()]
gates = [quclif.Gate(g) for g in gates]


ecc_set = quclif.ECCs.generate(
    nqubits=5,
    gates=gates,
    max_size=6,
)
print(ecc_set)
ecc_set.dump_quartz("eccset.json")
