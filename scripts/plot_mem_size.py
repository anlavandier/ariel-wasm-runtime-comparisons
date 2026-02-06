import matplotlib.pyplot as plt
import numpy as np

def plot_size(paths: list[(str, str)]):
    size: dict[str, dict[str, int]] = {}
    runtimes: set[str] = set()
    for hardware, file in paths:
        with open(file) as f:
            for l in f.readlines()[1:]:
                # runtime, .text, .data, .rodata, file
                runtime, text, _data, rodata, file_size = l.split(',')[:5]
                try:
                    size[hardware][runtime] = int(text) + int(rodata) - int(file_size)
                except KeyError:
                    size[hardware] = { runtime: int(text)  + int(rodata) - int(file_size) }

                runtimes.add(runtime)

    _fig, ax = plt.subplots()
    size = dict(sorted([(key, dict(sorted(val.items()))) for (key, val) in size.items()]))
    runtimes = list(sorted(runtimes))
    runtimes.remove("wasm-interpreter")
    runtimes.append("wasm-interpreter")
    n_runtimes = len(runtimes)
    cur_runtime = runtimes[0]

    ax.bar(
        x = np.arange(len(size)),
        height = [
            size_of_hardware.get(cur_runtime, 0) for
            size_of_hardware in size.values()
        ],
        width = 1./(n_runtimes + 1),
        label = cur_runtime,
        tick_label = list(size.keys()),
    )

    for i, cur_runtime in enumerate(runtimes[1:]):
        ax.bar(
            x = np.arange(len(size)) + (i + 1)/(n_runtimes + 1),
            height = [
                size_of_hardware.get(cur_runtime, 0) for
                size_of_hardware in size.values()
            ],
            width=1./(n_runtimes + 1),
            label = cur_runtime,
        )
    ax.tick_params(axis = 'x', labelrotation=45)
    ax.set_ylabel("Peak RAM usage memory size (bytes)")
    ax.legend()

    plt.show()

if __name__ == "__main__":
    import sys
    plot_size(paths=[(sys.argv[i], sys.argv[i+1]) for i in range(1, len(sys.argv)-1, 2)])