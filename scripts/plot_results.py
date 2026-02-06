import matplotlib.pyplot as plt
import numpy as np
import sys

def plot_benchmark(benchmark: str, paths: list[(str, str)], board: str):
    # Geometric Mean + geometric standard dev
    scores: dict[str, dict[str, tuple[float, float]]] = {}
    times: dict[str, dict[str, tuple[float, float]]] = {}
    runtime_names: list[str] = []
    for (runtime_name, file_name) in paths:
        with open(file_name) as f:
            for line in f.readlines():
                if len(line) == 0 or line[0] == "#":
                    continue
                # benchmark_name, score, score_dev, timing, dev
                splitted: list[str] = line.split(',')
                assert len(splitted) == 5
                name: str = splitted[0]
                score_m: float = float(splitted[1])
                score_std: float = float(splitted[2])
                times_m: float = float(splitted[3])
                times_std: float = float(splitted[4])

                try:
                    scores[name][runtime_name] = (score_m, score_std)
                except KeyError:
                    scores[name] = { runtime_name: (score_m, score_std) }

                try:
                    times[name][runtime_name] = (times_m, times_std)
                except KeyError:
                    times[name] = { runtime_name: (times_m, times_std) }

        runtime_names.append(runtime_name)

    _fig_score, ax_score = plt.subplots()
    scores = dict(sorted(scores.items()))
    times = dict(sorted(times.items()))
    n_runtimes = len(runtime_names)

    # Embench Scores
    cur_runtime = runtime_names[0]
    ax_score.bar(
        x = np.arange(len(scores)),
        height = [score_of_b.get(cur_runtime, (0, 0))[0] for score_of_b in scores.values()],
        width = 1./(n_runtimes + 1),
        label = cur_runtime,
        tick_label = list(scores.keys()),
        yerr = np.array(
            [[score_m - score_m/score_std, score_m * score_std - score_m]
                for (score_m, score_std) in [s.get(cur_runtime, (0, 1)) for s in scores.values()]
            ]
        ).transpose()
    )

    for i, cur_runtime in enumerate(runtime_names[1:]):
        ax_score.bar(
            x = np.arange(len(scores)) + (i + 1)/(n_runtimes + 1),
            height = [score_of_b.get(cur_runtime, (0, 0))[0] for score_of_b in scores.values()],
            width = 1./(n_runtimes + 1),
            label = cur_runtime,
            yerr = np.array(
                [[score_m - score_m/score_std, score_m * score_std - score_m]
                    for (score_m, score_std) in [s.get(cur_runtime, (0, 1)) for s in scores.values()]
                ]
            ).transpose()
        )

    ax_score.tick_params(axis='x', labelrotation=45)
    ax_score.set_ylabel(f"{benchmark} score")
    ax_score.legend()

    _fig_time, ax_time = plt.subplots()

    # Absolute execution time
    cur_runtime = runtime_names[0]
    ax_time.bar(
        x = np.arange(len(times)),
        height = [time_of_b.get(cur_runtime, (0, 0))[0] for time_of_b in times.values()],
        width = 1./(n_runtimes + 1),
        label = runtime_names[0],
        tick_label = list(times.keys()),
        yerr = np.array(
            [[times_m - times_m/times_std, times_m * times_std - times_m]
                for (times_m, times_std) in [t.get(cur_runtime, (0, 1)) for t in times.values()]
            ]
        ).transpose()
    )

    for i, cur_runtime in enumerate(runtime_names[1:]):
        ax_time.bar(
            x = np.arange(len(times)) + (i + 1)/(n_runtimes + 1),
            height = [time_of_b.get(cur_runtime, (0, 0))[0] for time_of_b in times.values()],
            width = 1./(n_runtimes + 1),
            label = cur_runtime,
            yerr = np.array(
                [[times_m - times_m/times_std, times_m * times_std - times_m]
                    for (times_m, times_std) in [t.get(cur_runtime, (0, 1)) for t in times.values()]
                ]
            ).transpose()
        )
    ax_time.set_ylabel("Time (ms)")
    ax_time.tick_params(axis='x', labelrotation=45)
    ax_time.legend()

    plt.show()



if __name__ == "__main__":
    # benchmark board, (runtime file)*
    args: list[str] = sys.argv[1:]
    benchmark_name = args[0]
    board = args[1]
    annotated_files = [(args[i], args[i+1]) for i in range(2, len(args), 2)]
    print(annotated_files)
    plot_benchmark(benchmark=benchmark_name, paths=annotated_files, board=board)