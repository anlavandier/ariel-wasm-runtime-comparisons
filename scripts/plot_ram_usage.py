import matplotlib.pyplot as plt
import numpy as np
import sys

def plot_benchmark(benchmark: str, paths: list[(str, str)], board: str):
    peak_ram_usage: dict[str, dict[str, int]] = {}
    runtimes: set[str] = []
    for (runtime_name, file_name) in paths:
        with open(file_name) as f:
            for line in f.readlines():
                # benchmark_name, peak heap, .data, .bss
                if len(line) == 0 or line[0] == "#": continue
                splitted: list[str] = line.split(',')
                assert len(splitted) >= 4
                name: str = splitted[0]
                peak_usage = int(splitted[1]) + int(splitted[2]) + int(splitted[3])
                try:
                    peak_ram_usage[name][runtime_name] = peak_usage
                except KeyError:
                    peak_ram_usage[name] = { runtime_name: peak_usage }


        runtimes.append(runtime_name)

    _fig, ax = plt.subplots()

    peak_ram_usage = dict(sorted([(key, dict(sorted(val.items()))) for (key, val) in peak_ram_usage.items()]))
    runtimes = list(sorted(runtimes))
    # Keep order consistent
    try:
        runtimes.remove("wasm-interpreter")
        runtimes.append("wasm-interpreter")
    except ValueError:
        pass


    n_runtimes = len(runtimes)
    cur_runtime = runtimes[0]

    ax.bar(
        x = np.arange(len(peak_ram_usage)),
        height = [
            size_of_hardware.get(cur_runtime, 0) for
            size_of_hardware in peak_ram_usage.values()
        ],
        width = 1./(n_runtimes + 1),
        label = cur_runtime,
        tick_label = list(peak_ram_usage.keys()),
    )

    for i, cur_runtime in enumerate(runtimes[1:]):
        ax.bar(
            x = np.arange(len(peak_ram_usage)) + (i + 1)/(n_runtimes + 1),
            height = [
                size_of_hardware.get(cur_runtime, 0) for
                size_of_hardware in peak_ram_usage.values()
            ],
            width=1./(n_runtimes + 1),
            label = cur_runtime,
        )



    ax.hlines(
        y = 64 * 1024 * 2,
        xmin=-0.2,
        xmax=len(peak_ram_usage),
        linestyles="dotted",
        label="Wasm Linear Memory",
        colors="black"
    )
    ax.tick_params(axis = 'x', labelrotation=45)
    ax.set_ylabel("Peak RAM usage (bytes)")
    ax.legend()
    plt.show()



if __name__ == "__main__":
    # benchmark board, (runtime file)*
    args: list[str] = sys.argv[1:]
    benchmark_name = args[0]
    board = args[1]
    annotated_files = [(args[i], args[i+1]) for i in range(2, len(args), 2)]
    print(annotated_files)
    plot_benchmark(benchmark=benchmark_name, paths=annotated_files, board=board)