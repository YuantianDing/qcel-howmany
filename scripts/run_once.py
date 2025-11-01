import os
import quclif
from build_prover import build_prover

import sys

from view_rules import show_rules_pdf

if __name__ == "__main__":
    gate_set_name = sys.argv[1]
    ngates = int(sys.argv[2])
    file = f".cache/prover-{gate_set_name}-{ngates}-5.prover.json"
    if not os.path.exists(file):
        print(f"Building prover for gate set '{gate_set_name}' with {ngates} gates.")
        build_prover(gate_set_name, ngates=ngates)
    show_rules_pdf(file)