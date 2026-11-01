# Benchmark results

SIMULATED fixtures. One machine; see environment.json. Medians with interquartile range.

## B1: numerical representation

| fixture | record | stable identity | compact bytes | parse | serialize | values differing from recorded decimal |
|---|---|---|---|---|---|---|
| short-dense-unchanged | binary64 | true | 38312 | 176.9 µs (IQR 24.4) | 44.9 µs (IQR 0.6) | 1500 / 2500 |
| short-dense-unchanged | exact | true | 48278 | 503.4 µs (IQR 56.9) | 33.7 µs (IQR 0.4) | 0 / 2500 |
| short-dense-changing | binary64 | true | 44330 | 293.2 µs (IQR 102.7) | 94.4 µs (IQR 22.6) | 1983 / 2500 |
| short-dense-changing | exact | true | 49455 | 799.3 µs (IQR 78.7) | 43.9 µs (IQR 1.2) | 0 / 2500 |
| short-sparse-unchanged | binary64 | true | 19268 | 71.5 µs (IQR 23.2) | 18.2 µs (IQR 5.2) | 172 / 306 |
| short-sparse-unchanged | exact | true | 20439 | 95.7 µs (IQR 36.8) | 12.8 µs (IQR 1.1) | 0 / 306 |
| short-sparse-changing | binary64 | true | 19925 | 67.5 µs (IQR 11.6) | 25.7 µs (IQR 7.5) | 236 / 306 |
| short-sparse-changing | exact | true | 20524 | 120.7 µs (IQR 27.5) | 17.8 µs (IQR 1.1) | 0 / 306 |
| long-dense-unchanged | binary64 | true | 379812 | 3.86 ms (IQR 0.31) | 576.6 µs (IQR 84.9) | 15000 / 25000 |
| long-dense-unchanged | exact | true | 479778 | 6.11 ms (IQR 1.69) | 424.3 µs (IQR 72.2) | 0 / 25000 |
| long-dense-changing | binary64 | true | 443190 | 2.20 ms (IQR 0.18) | 931.7 µs (IQR 147.0) | 19804 / 25000 |
| long-dense-changing | exact | true | 494298 | 6.50 ms (IQR 0.47) | 437.1 µs (IQR 87.5) | 0 / 25000 |
| long-sparse-unchanged | binary64 | true | 190149 | 621.6 µs (IQR 66.0) | 187.7 µs (IQR 52.9) | 1904 / 3154 |
| long-sparse-unchanged | exact | true | 202885 | 1.07 ms (IQR 0.09) | 182.8 µs (IQR 17.1) | 0 / 3154 |
| long-sparse-changing | binary64 | true | 197523 | 629.3 µs (IQR 110.3) | 246.7 µs (IQR 36.8) | 2533 / 3154 |
| long-sparse-changing | exact | true | 204035 | 1.23 ms (IQR 0.19) | 160.5 µs (IQR 26.9) | 0 / 3154 |

Per exact value (ns, median): see results.json `per_value_ns`.

## B2: field reconstruction (all frames)

| fixture | values | field_at every frame | fields single pass | ratio |
|---|---|---|---|---|
| short-dense-unchanged | binary64 | 27.58 ms (IQR 0.50) | 218.0 µs (IQR 43.5) | 126.5x |
| short-dense-unchanged | exact_owned | 67.98 ms (IQR 1.48) | 629.4 µs (IQR 152.5) | 108.0x |
| short-dense-unchanged | exact_shared | 34.05 ms (IQR 0.78) | 514.3 µs (IQR 61.8) | 66.2x |
| short-dense-changing | binary64 | 27.57 ms (IQR 0.75) | 210.4 µs (IQR 18.9) | 131.0x |
| short-dense-changing | exact_owned | 88.34 ms (IQR 1.69) | 488.9 µs (IQR 120.1) | 180.7x |
| short-dense-changing | exact_shared | 33.59 ms (IQR 1.42) | 286.6 µs (IQR 47.6) | 117.2x |
| short-sparse-unchanged | binary64 | 3.99 ms (IQR 0.25) | 116.2 µs (IQR 24.3) | 34.3x |
| short-sparse-unchanged | exact_owned | 9.51 ms (IQR 0.42) | 190.4 µs (IQR 49.1) | 50.0x |
| short-sparse-unchanged | exact_shared | 4.93 ms (IQR 0.29) | 180.9 µs (IQR 19.3) | 27.3x |
| short-sparse-changing | binary64 | 4.11 ms (IQR 0.25) | 121.6 µs (IQR 35.2) | 33.8x |
| short-sparse-changing | exact_owned | 11.48 ms (IQR 0.55) | 168.2 µs (IQR 18.9) | 68.2x |
| short-sparse-changing | exact_shared | 4.76 ms (IQR 0.26) | 159.0 µs (IQR 35.9) | 29.9x |
| long-dense-unchanged | binary64 | 2893.48 ms (IQR 20.40) | 3.32 ms (IQR 0.75) | 871.5x |
| long-dense-unchanged | exact_owned | 7112.20 ms (IQR 16.81) | 5.54 ms (IQR 0.66) | 1283.9x |
| long-dense-unchanged | exact_shared | 3506.99 ms (IQR 10.69) | 3.50 ms (IQR 0.34) | 1001.3x |
| long-dense-changing | binary64 | 2863.83 ms (IQR 6.33) | 2.87 ms (IQR 0.53) | 997.3x |
| long-dense-changing | exact_owned | 8947.98 ms (IQR 12.37) | 6.33 ms (IQR 0.50) | 1414.6x |
| long-dense-changing | exact_shared | 3480.05 ms (IQR 2.76) | 3.73 ms (IQR 0.31) | 932.0x |
| long-sparse-unchanged | binary64 | 454.43 ms (IQR 4.08) | 1.70 ms (IQR 0.22) | 267.0x |
| long-sparse-unchanged | exact_owned | 1019.87 ms (IQR 2.18) | 2.27 ms (IQR 0.25) | 448.7x |
| long-sparse-unchanged | exact_shared | 536.89 ms (IQR 0.92) | 1.96 ms (IQR 0.29) | 274.2x |
| long-sparse-changing | binary64 | 451.55 ms (IQR 3.31) | 1.91 ms (IQR 0.19) | 236.9x |
| long-sparse-changing | exact_owned | 1282.51 ms (IQR 6.82) | 2.25 ms (IQR 0.32) | 569.2x |
| long-sparse-changing | exact_shared | 541.80 ms (IQR 3.85) | 1.94 ms (IQR 0.16) | 279.0x |

## B3: continuous relational state (all frames)

| fixture | values | retained: advance only | retained: process changes | retained: + snapshots | forward fill: build | forward fill: snapshots | fields single pass |
|---|---|---|---|---|---|---|---|
| short-dense-unchanged | binary64 | 0.6 µs (IQR 0.1) | 31.2 µs (IQR 2.4) | 227.5 µs (IQR 49.7) | 57.5 µs (IQR 14.0) | 230.2 µs (IQR 49.2) | 414.5 µs (IQR 171.0) |
| short-dense-unchanged | exact_shared | 0.6 µs (IQR 0.1) | 53.5 µs (IQR 11.4) | 314.0 µs (IQR 46.7) | 102.1 µs (IQR 1.5) | 223.8 µs (IQR 31.2) | 318.8 µs (IQR 92.6) |
| short-dense-changing | binary64 | 0.7 µs (IQR 0.0) | 35.3 µs (IQR 0.7) | 245.3 µs (IQR 34.0) | 60.1 µs (IQR 13.5) | 207.9 µs (IQR 50.2) | 226.5 µs (IQR 49.3) |
| short-dense-changing | exact_shared | 0.7 µs (IQR 0.1) | 48.5 µs (IQR 12.2) | 309.5 µs (IQR 54.1) | 127.2 µs (IQR 25.7) | 214.7 µs (IQR 41.8) | 305.7 µs (IQR 53.6) |
| short-sparse-unchanged | binary64 | 0.7 µs (IQR 0.1) | 6.6 µs (IQR 0.1) | 217.6 µs (IQR 37.6) | 41.3 µs (IQR 0.8) | 212.7 µs (IQR 47.5) | 123.1 µs (IQR 14.5) |
| short-sparse-unchanged | exact_shared | 0.8 µs (IQR 0.1) | 9.8 µs (IQR 0.6) | 212.9 µs (IQR 45.9) | 58.8 µs (IQR 5.1) | 217.6 µs (IQR 46.7) | 163.2 µs (IQR 29.4) |
| short-sparse-changing | binary64 | 0.7 µs (IQR 0.1) | 6.5 µs (IQR 0.3) | 211.8 µs (IQR 55.8) | 35.4 µs (IQR 1.1) | 192.7 µs (IQR 47.3) | 119.7 µs (IQR 58.6) |
| short-sparse-changing | exact_shared | 0.5 µs (IQR 0.1) | 7.6 µs (IQR 0.2) | 221.8 µs (IQR 88.9) | 71.5 µs (IQR 14.8) | 218.5 µs (IQR 106.6) | 150.3 µs (IQR 29.1) |
| long-dense-unchanged | binary64 | 1.8 µs (IQR 0.1) | 335.8 µs (IQR 85.0) | 2.57 ms (IQR 0.16) | 696.2 µs (IQR 73.0) | 2.15 ms (IQR 0.25) | 2.72 ms (IQR 0.18) |
| long-dense-unchanged | exact_shared | 2.5 µs (IQR 0.4) | 640.0 µs (IQR 124.2) | 3.06 ms (IQR 0.26) | 1.32 ms (IQR 0.13) | 2.36 ms (IQR 0.16) | 3.37 ms (IQR 0.34) |
| long-dense-changing | binary64 | 1.5 µs (IQR 0.1) | 383.1 µs (IQR 87.4) | 2.63 ms (IQR 0.16) | 660.3 µs (IQR 70.6) | 2.12 ms (IQR 0.08) | 2.77 ms (IQR 0.21) |
| long-dense-changing | exact_shared | 1.4 µs (IQR 0.1) | 662.5 µs (IQR 111.8) | 2.99 ms (IQR 0.22) | 1.29 ms (IQR 0.10) | 2.29 ms (IQR 0.25) | 3.62 ms (IQR 0.29) |
| long-sparse-unchanged | binary64 | 1.7 µs (IQR 0.1) | 75.7 µs (IQR 3.3) | 2.24 ms (IQR 0.09) | 400.8 µs (IQR 80.7) | 2.18 ms (IQR 0.20) | 1.73 ms (IQR 0.24) |
| long-sparse-unchanged | exact_shared | 1.8 µs (IQR 0.0) | 127.0 µs (IQR 37.5) | 2.50 ms (IQR 0.28) | 717.9 µs (IQR 94.2) | 2.21 ms (IQR 0.17) | 1.86 ms (IQR 0.15) |
| long-sparse-changing | binary64 | 1.8 µs (IQR 0.1) | 80.3 µs (IQR 14.8) | 2.20 ms (IQR 0.20) | 379.2 µs (IQR 42.5) | 2.11 ms (IQR 0.11) | 1.49 ms (IQR 0.24) |
| long-sparse-changing | exact_shared | 1.8 µs (IQR 0.1) | 124.6 µs (IQR 32.2) | 2.48 ms (IQR 0.22) | 769.7 µs (IQR 133.9) | 2.30 ms (IQR 0.11) | 2.35 ms (IQR 1.00) |

## Kernel consistency

| fixture | implementations compared | identical | digest |
|---|---|---|---|
| long-dense-changing | 5 | true | `d38dfe0292ddbe80` |
| long-dense-unchanged | 5 | true | `b71f96ded3b0aabe` |
| long-sparse-changing | 5 | true | `cd1d40757124cefd` |
| long-sparse-unchanged | 5 | true | `b71f96ded3b0aabe` |
| short-dense-changing | 5 | true | `23992eb9777489b7` |
| short-dense-unchanged | 5 | true | `90029c5d6fa04322` |
| short-sparse-changing | 5 | true | `2880d91501878e56` |
| short-sparse-unchanged | 5 | true | `90029c5d6fa04322` |
