import os
import qcel_howmany
from build_prover import build_prover

import sys

from prove import prove
from view_rules import show_rules_pdf

if __name__ == "__main__":
    gate_set_name = sys.argv[1]
    ngates = int(sys.argv[2])
    if '--naive' in sys.argv:
        file = f".cache/prove-{gate_set_name}-{ngates}-5-naive.json"
    else:
        file = f".cache/prover-{gate_set_name}-{ngates}-5.prover.json"
    if not os.path.exists(file):
        prover, _ = build_prover(gate_set_name, ngates=ngates)
        if '--naive' in sys.argv:
            prove(prover, gate_set_name, ngates=ngates, naive=True)

    show_rules_pdf(file)