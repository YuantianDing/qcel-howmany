


from quclif import Gate


GATE_SETS = {
    "logical": ["X", "CX"],
    "clifford": ["H", "S", "SDG", "X", "CX"],
    "clifford-t": ["H", "T", "TDG", "X", "CX"],
    "common-clifford-t": ["H", "S", "T", "TDG", "X", "CX", "CY", "CZ", "Y", "Z", "S", "SDG"],
}

for v in GATE_SETS.values():
    for g in v:
        Gate(g)

