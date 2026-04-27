
import os
import qiskit
import qiskit.qasm2
from qiskit.circuit import library as qlib
import qiskit.quantum_info as qi
import qcel_howmany
import sys

with open("viewing.tmp", 'w') as f:
    for a in sys.argv[1:]:
        f.write(f"---------- Circuit {a} ----------\n")
        circ = qiskit.qasm2.load(a)
        f.write(str(circ.draw(output='text', fold=-1)) + "\n")

os.system("vim viewing.tmp -c 'set nowrap'")

os.remove("viewing.tmp")
