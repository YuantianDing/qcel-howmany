import json
import os
import sys
import pandas as pd
from run_all import NGATES
from gate_set import GATE_SETS

def load(a: str):
    if os.path.exists(a):
        with open(a) as f:
            return json.load(f)
    return {}

def retrieve_data():
    data = {}
    for gate_set_name, (gate_count, prove_gate_count, naive_gate_count) in NGATES.items():
        data[gate_set_name] = {i: {} for i in range(1, prove_gate_count + 1)}
        for ngates in range(1, prove_gate_count + 1):
            gateset = gate_set_name.replace("/","::")
            data[gate_set_name][ngates]['eccset'] = load(f".cache/eccset-{gateset}-{ngates}-5.eccs.json")
        for ngates in range(1, gate_count + 1):
            gateset = gate_set_name.replace("/","::")
            data[gate_set_name][ngates]['prove'] = load(f".cache/prover-{gateset}-{ngates}-5.prover.json")
        for ngates in range(gate_count + 1, prove_gate_count + 1):
            gateset = gate_set_name.replace("/","::")
            data[gate_set_name][ngates]['prove'] = load(f".cache/prove-{gateset}-{ngates}-5.json")
        for ngates in range(1, naive_gate_count + 1):
            gateset = gate_set_name.replace("/","::")
            data[gate_set_name][ngates]['eccset-naive'] = load(f".cache/eccset-{gateset}-{ngates}-5-naive.eccs.json")
        for ngates in range(1, naive_gate_count + 1):
            gateset = gate_set_name.replace("/","::")
            data[gate_set_name][ngates]['prove-naive'] = load(f".cache/prove-{gateset}-{ngates}-5-naive.json")
        for ngates in range(1, naive_gate_count + 1):
            gateset = gate_set_name.replace("/","::")
            data[gate_set_name][ngates]['prover-naive'] = load(f".cache/prover-{gateset}-{ngates}-5-naive.prover.json")
    return data

def get_dataframe(f):
    data = retrieve_data()
    length = max(i for v in data.values() for i in v.keys())
    records = []
    for i in range(1, length + 1):
        d = {}
        for gate_set_name, gate_sets in data.items():
            d[gate_set_name] = gate_sets.get(i, '')
            if d[gate_set_name] != '':
                d[gate_set_name] = f(d[gate_set_name])
        # print(d)
        records.append(d)
    df = pd.DataFrame.from_records(records, coerce_float=False)
    df.index += 1
    return df

if __name__ == "__main__":
    pd.set_option('display.max_colwidth', None)
    pd.set_option('display.max_columns', None)
    if '--naive' in sys.argv:
        a = {
            "[Naive] Number of Rules": lambda x: len(x['prove-naive']['rules']) if 'prove-naive' in x and 'rules' in x['prove-naive'] else 'w',
            "[Naive] Proving Time (s)": lambda x: x['prove-naive']['time'] / 1e9 if 'prove-naive' in x and 'time' in x['prove-naive'] else 'w',
            "[Naive] Synthesis Time (s)": lambda x: x['eccset-naive']['time'] / 1e9 if 'eccset-naive' in x and 'time' in x['eccset-naive'] else 'w',
            "[Naive] Synthesis Result Checking Time (s)": lambda x: x['eccset-naive']['checking_time'] / 1e9 if 'eccset-naive' in x and 'checking_time' in x['eccset-naive'] else 'w',
            "[Naive] Synthesized ECCs": lambda x: x['eccset-naive']['eccs'] if 'eccset-naive' in x and 'eccs' in x['eccset-naive'] else 'w',
            "[Naive] Synthesized Identities": lambda x: x['eccset-naive']['identities'] if 'eccset-naive' in x and 'identities' in x['eccset-naive'] else 'w',
        }
    else:
        a = {
            "Number of Rules": lambda x: len(x['prove']['rules']) if 'prove' in x and 'rules' in x['prove'] else 'w',
            "Proving Time (s)": lambda x: x['prove']['time'] / 1e9 if 'prove' in x and 'time' in x['prove'] else 'w',
            "Synthesis Time (s)": lambda x: x['eccset']['time'] / 1e9 if 'eccset' in x and 'time' in x['eccset'] else 'w',
            "Synthesized ECCs": lambda x: x['eccset']['eccs'] if 'eccset' in x and 'eccs' in x['eccset'] else 'w',
            "Synthesized Identities": lambda x: x['eccset']['identities'] if 'eccset' in x and 'identities' in x['eccset'] else 'w',
        }
    if '--latex' in sys.argv:
        for name, func in a.items():
            text = get_dataframe(func).to_latex(caption=name, float_format="%.2f")
            text = text.replace(" logical ",  r"\classical")
            text = text.replace(" common-clifford-t ", r"\commoncliffordt")
            text = text.replace(" clifford-t1/2 ", r"\cliffordthalf")
            text = text.replace(" clifford-rz(pi/3) ", r"\cliffordrz")
            text = text.replace(" clifford-t ", r"\cliffordt")
            text = text.replace(" clifford ", r"\clifford")
            text = text.replace(r"\toprule", r"\toprule $k$")

            print(text, end="\n\n")
            print()
            print()
    else:
        for name, func in a.items():
            print("---------------------------------------------------------------------------")
            print(f"{name}:\n")
            print(get_dataframe(func), end="\n\n")

        print("---------------------------------------------------------------------------")
        print(f"Gate Set Definition:\n")

        for gate_set_name, gate_count in GATE_SETS.items():
            print(f"{gate_set_name} ({len(gate_count)} gates): {', '.join(GATE_SETS[gate_set_name])}")
        print()

