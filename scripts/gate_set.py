


from quclif import Gate


GATE_SETS = {
    "logical": ["X", "CX"],
    "clifford": ["H", "S", "SDG", "X", "CX"],
    "clifford-t": ["H", "T", "TDG", "X", "CX"],
    "common-clifford-t": ["H", "S", "T", "TDG", "X", "CX", "CY", "CZ", "Y", "Z", "S", "SDG"],
    "clifford-t1/2": ["H", "T1/2", "TDG1/2", "S", "SDG", "X", "CX"],
    "clifford-rz(pi/3)": ["H", "S", "SDG", "X", "CX", "RZ(pi/3)", "RZ(-pi/3)"],
}

for v in GATE_SETS.values():
    for g in v:
        Gate(g)

