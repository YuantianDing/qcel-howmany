


import os
import subprocess
import pandas as pd


def run_synthesis(percision_level = 24, f128 = False):
    commands = ["cargo", "run", "--release", "--bin", "run_synthesis_fp"]

    if f128:
        commands.append("-Ff128")
    
    print(f"Compiling PERCISION_LEVEL={percision_level} {'f128' if f128 else 'f64'}", end=' ', flush=True)
    os.environ["PERCISION_LEVEL"] = str(percision_level)
    process = subprocess.run(commands, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    del os.environ["PERCISION_LEVEL"]

    result = int(process.stdout.strip().split("\n")[-1])
    print(f"=> {result}")
    return result

if __name__ == "__main__":
    with pd.HDFStore('.cache/fixed_point_results.h5') as store:
        if 'df' in store:
            df = store['df']
        else:
            data = []
            for i in [8, 16, 24, 32, 40, 48]:
                data.append({
                    "f64": run_synthesis(i),
                    "f128": run_synthesis(i, f128=True)
                })
            
            df = pd.DataFrame(data, index=[8, 16, 24, 32, 40, 48])
            store['df'] = df
        print(df.to_latex())
