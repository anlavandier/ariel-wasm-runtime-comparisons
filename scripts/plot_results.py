import matplotlib.pyplot as plt
import numpy as np
import sys

def plot_benchmark(benchmark: str, paths: list[(str, str)], board: str):
    # Geometric Mean + geometric standard dev
    scores: dict[str, list[float, float]] = {}
    times: dict[str, list[float, float]] = {}
    runtime_names: list[str] = []
    for (runtime_name, file_name) in paths:
        with open(file_name) as f:
            for line in f.readlines():
                # benchmark_name, score, score_dev, timing, dev
                splitted: list[str] = line.split(',')
                assert len(splitted) == 5
                name: str = splitted[0]
                score_m: float = float(splitted[1])
                score_std: float = float(splitted[2])
                times_m: float = float(splitted[3])
                times_std: float = float(splitted[4])

                try:
                    scores[name].append((score_m, score_std))
                except KeyError:
                    scores[name] = [(score_m, score_std)]

                try:
                    times[name].append((times_m, times_std))
                except KeyError:
                    times[name] = [(times_m, times_std)]

        runtime_names.append(runtime_name)

    _fig_score, ax_score = plt.subplots()

    n_runtimes = len(runtime_names)

    ax_score.bar(
        x = np.arange(len(scores)),
        height = [score_of_b[0][0] for score_of_b in scores.values()],
        width = 1./(n_runtimes + 1),
        label = runtime_names[0],
        tick_label = list(scores.keys()),
        yerr = np.array(
            [[score_m - score_m/score_std, score_m * score_std - score_m]
                for (score_m, score_std) in [s[0] for s in scores.values()]
            ]
        ).transpose()
    )

    for i, r in enumerate(runtime_names[1:]):
        ax_score.bar(
            x = np.arange(len(scores)) + (i + 1)/(n_runtimes + 1),
            height = [score_of_b[i + 1][0] for score_of_b in scores.values()],
            width = 1./(n_runtimes + 1),
            label = r,
            yerr = np.array(
                [[score_m - score_m/score_std, score_m * score_std - score_m]
                    for (score_m, score_std) in [s[i + 1] for s in scores.values()]
                ]
            ).transpose()
        )

    ax_score.tick_params(axis='x', labelrotation=45)
    ax_score.set_ylabel(f"{benchmark} score")
    ax_score.set_title(f"{benchmark} scores comparisons on {board}\nHigher is better")
    ax_score.legend()

    _fig_time, ax_time = plt.subplots()

    ax_time.bar(
        x = np.arange(len(times)),
        height = [time_of_b[0][0] for time_of_b in times.values()],
        width = 1./(n_runtimes + 1),
        label = runtime_names[0],
        tick_label = list(times.keys()),
        yerr = np.array(
            [[times_m - times_m/times_std, times_m * times_std - times_m]
                for (times_m, times_std) in [t[0] for t in times.values()]
            ]
        ).transpose()
    )

    for i, r in enumerate(runtime_names[1:]):
        ax_time.bar(
            x = np.arange(len(times)) + (i + 1)/(n_runtimes + 1),
            height = [time_of_b[i + 1][0] for time_of_b in times.values()],
            width = 1./(n_runtimes + 1),
            label = r,
            yerr = np.array(
                [[times_m - times_m/times_std, times_m * times_std - times_m]
                    for (times_m, times_std) in [t[i + 1] for t in times.values()]
                ]
            ).transpose()
        )
    ax_time.set_ylabel("Time (ms)")
    ax_time.tick_params(axis='x', labelrotation=45)
    ax_time.set_title(f"{benchmark} timing comparisons on {board}\n Lower is better")
    ax_time.legend()
    plt.show()



if __name__ == "__main__":
    # benchmark name, (runtime file)*
    args: list[str] = sys.argv[1:]
    benchmark_name = args[0]
    board = args[1]
    annotated_files = [(args[i], args[i+1]) for i in range(2, len(args), 2)]
    print(annotated_files)
    plot_benchmark(benchmark=benchmark_name, paths=annotated_files, board=board)