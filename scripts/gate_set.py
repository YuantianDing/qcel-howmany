


from qcel_howmany import Gate


GATE_SETS = {
    "logical": ["X", "CX"],
    "clifford": ["H", "S", "SDG", "X", "CX"],
    "clifford-t": ["H", "T", "TDG", "X", "CX"],
    "common-clifford-t": ["H", "S", "T", "TDG", "X", "CX", "CY", "CZ", "Y", "Z", "S", "SDG"],
    "clifford-t1/2": ["H", "T1/2", "TDG1/2", "S", "SDG", "X", "CX"],
    "clifford-rz(pi/3)": ["H", "S", "SDG", "X", "CX", "RZ(pi/3)", "RZ(-pi/3)"],
    "clifford-rz(pi/5)": ["H", "S", "SDG", "X", "CX", "RZ(pi/5)", "RZ(-pi/5)"],
    "clifford-rz(pi/16)": ["H", "X", "CX", "T", "TDG", "RZ(pi/16)", "RZ(-pi/16)"],
}

for v in GATE_SETS.values():
    for g in v:
        Gate(g)

