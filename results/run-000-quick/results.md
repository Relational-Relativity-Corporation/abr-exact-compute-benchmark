# Benchmark results

SIMULATED fixtures. One machine; see environment.json. Medians with interquartile range.

## B1: numerical representation

| fixture | record | stable identity | compact bytes | parse | serialize | values differing from recorded decimal |
|---|---|---|---|---|---|---|
| short-dense-unchanged | binary64 | true | 38312 | 176.1 µs (IQR 37.3) | 55.4 µs (IQR 2.4) | 1500 / 2500 |
| short-dense-unchanged | exact | true | 48278 | 531.3 µs (IQR 61.7) | 38.2 µs (IQR 0.5) | 0 / 2500 |
| short-dense-changing | binary64 | true | 44330 | 343.0 µs (IQR 97.8) | 100.0 µs (IQR 0.6) | 1983 / 2500 |
| short-dense-changing | exact | true | 49455 | 710.1 µs (IQR 81.2) | 39.1 µs (IQR 0.6) | 0 / 2500 |
| short-sparse-unchanged | binary64 | true | 19268 | 55.5 µs (IQR 1.6) | 17.5 µs (IQR 0.2) | 172 / 306 |
| short-sparse-unchanged | exact | true | 20439 | 101.8 µs (IQR 23.7) | 15.5 µs (IQR 0.9) | 0 / 306 |
| short-sparse-changing | binary64 | true | 19925 | 57.8 µs (IQR 3.1) | 22.5 µs (IQR 0.9) | 236 / 306 |
| short-sparse-changing | exact | true | 20524 | 94.5 µs (IQR 5.7) | 16.2 µs (IQR 0.3) | 0 / 306 |
| long-dense-unchanged | binary64 | true | 379812 | 4.75 ms (IQR 0.24) | 1.19 ms (IQR 0.39) | 15000 / 25000 |
| long-dense-unchanged | exact | true | 479778 | 8.29 ms (IQR 0.48) | 470.0 µs (IQR 275.6) | 0 / 25000 |
| long-dense-changing | binary64 | true | 443190 | 4.25 ms (IQR 0.10) | 1.41 ms (IQR 0.16) | 19804 / 25000 |
| long-dense-changing | exact | true | 494298 | 8.42 ms (IQR 0.79) | 1.11 ms (IQR 0.14) | 0 / 25000 |
| long-sparse-unchanged | binary64 | true | 190149 | 887.0 µs (IQR 173.6) | 223.8 µs (IQR 8.2) | 1904 / 3154 |
| long-sparse-unchanged | exact | true | 202885 | 1.32 ms (IQR 0.22) | 265.8 µs (IQR 40.2) | 0 / 3154 |
| long-sparse-changing | binary64 | true | 197523 | 564.4 µs (IQR 46.9) | 277.5 µs (IQR 44.0) | 2533 / 3154 |
| long-sparse-changing | exact | true | 204035 | 1.20 ms (IQR 0.20) | 169.1 µs (IQR 3.2) | 0 / 3154 |

Per exact value (ns, median): see results.json `per_value_ns`.

## B2: field reconstruction (all frames)

| fixture | values | field_at every frame | fields single pass | ratio |
|---|---|---|---|---|
| short-dense-unchanged | binary64 | 27.45 ms (IQR 0.32) | 491.8 µs (IQR 55.9) | 55.8x |
| short-dense-unchanged | exact_owned | 65.60 ms (IQR 0.39) | 641.3 µs (IQR 65.1) | 102.3x |
| short-dense-unchanged | exact_shared | 32.93 ms (IQR 0.71) | 502.8 µs (IQR 55.0) | 65.5x |
| short-dense-changing | binary64 | 26.72 ms (IQR 0.31) | 246.7 µs (IQR 33.8) | 108.3x |
| short-dense-changing | exact_owned | 92.51 ms (IQR 1.35) | 639.9 µs (IQR 35.9) | 144.6x |
| short-dense-changing | exact_shared | 33.67 ms (IQR 0.31) | 375.8 µs (IQR 72.8) | 89.6x |
| short-sparse-unchanged | binary64 | 4.19 ms (IQR 0.24) | 413.3 µs (IQR 69.4) | 10.1x |
| short-sparse-unchanged | exact_owned | 10.25 ms (IQR 0.19) | 220.5 µs (IQR 60.5) | 46.5x |
| short-sparse-unchanged | exact_shared | 4.73 ms (IQR 0.16) | 204.1 µs (IQR 48.6) | 23.2x |
| short-sparse-changing | binary64 | 3.98 ms (IQR 0.21) | 321.9 µs (IQR 39.9) | 12.4x |
| short-sparse-changing | exact_owned | 11.38 ms (IQR 0.31) | 216.9 µs (IQR 26.6) | 52.4x |
| short-sparse-changing | exact_shared | 5.12 ms (IQR 0.27) | 159.6 µs (IQR 26.5) | 32.1x |
| long-dense-unchanged | binary64 | 2867.73 ms (IQR 0.00) | 5.33 ms (IQR 0.29) | 538.3x |
| long-dense-unchanged | exact_owned | 6983.77 ms (IQR 0.00) | 8.99 ms (IQR 0.36) | 776.5x |
| long-dense-unchanged | exact_shared | 3491.99 ms (IQR 0.00) | 6.65 ms (IQR 0.33) | 525.2x |
| long-dense-changing | binary64 | 2862.47 ms (IQR 0.00) | 5.03 ms (IQR 0.31) | 568.6x |
| long-dense-changing | exact_owned | 9190.19 ms (IQR 0.00) | 9.32 ms (IQR 0.27) | 986.0x |
| long-dense-changing | exact_shared | 3543.62 ms (IQR 0.00) | 6.95 ms (IQR 0.40) | 510.2x |
| long-sparse-unchanged | binary64 | 453.75 ms (IQR 0.00) | 2.58 ms (IQR 0.27) | 175.8x |
| long-sparse-unchanged | exact_owned | 995.25 ms (IQR 0.00) | 2.53 ms (IQR 0.24) | 393.0x |
| long-sparse-unchanged | exact_shared | 533.67 ms (IQR 0.00) | 2.01 ms (IQR 0.11) | 265.1x |
| long-sparse-changing | binary64 | 451.56 ms (IQR 0.00) | 1.74 ms (IQR 0.10) | 260.2x |
| long-sparse-changing | exact_owned | 1236.74 ms (IQR 0.00) | 2.67 ms (IQR 0.53) | 463.7x |
| long-sparse-changing | exact_shared | 533.28 ms (IQR 0.00) | 3.16 ms (IQR 0.38) | 168.9x |

## B3: continuous relational state (all frames)

| fixture | values | retained: advance only | retained: process changes | retained: + snapshots | forward fill: build | forward fill: snapshots | fields single pass |
|---|---|---|---|---|---|---|---|
| short-dense-unchanged | binary64 | 0.7 µs (IQR 0.1) | 29.5 µs (IQR 0.4) | 223.4 µs (IQR 55.3) | 146.6 µs (IQR 39.0) | 231.3 µs (IQR 24.5) | 423.3 µs (IQR 16.2) |
| short-dense-unchanged | exact_shared | 0.7 µs (IQR 0.0) | 49.0 µs (IQR 0.5) | 259.3 µs (IQR 52.0) | 187.6 µs (IQR 50.9) | 202.2 µs (IQR 25.6) | 314.6 µs (IQR 51.9) |
| short-dense-changing | binary64 | 0.8 µs (IQR 0.2) | 35.1 µs (IQR 0.5) | 225.2 µs (IQR 3.3) | 141.3 µs (IQR 37.9) | 244.4 µs (IQR 25.1) | 304.2 µs (IQR 29.1) |
| short-dense-changing | exact_shared | 0.9 µs (IQR 0.1) | 61.2 µs (IQR 0.5) | 257.2 µs (IQR 8.0) | 179.4 µs (IQR 37.7) | 234.0 µs (IQR 15.2) | 346.0 µs (IQR 93.2) |
| short-sparse-unchanged | binary64 | 0.9 µs (IQR 0.2) | 6.1 µs (IQR 0.7) | 245.5 µs (IQR 4.1) | 133.1 µs (IQR 15.8) | 234.1 µs (IQR 22.9) | 133.8 µs (IQR 39.9) |
| short-sparse-unchanged | exact_shared | 0.7 µs (IQR 0.1) | 8.3 µs (IQR 1.1) | 211.0 µs (IQR 9.2) | 58.1 µs (IQR 1.9) | 199.7 µs (IQR 15.8) | 162.8 µs (IQR 5.3) |
| short-sparse-changing | binary64 | 0.9 µs (IQR 0.1) | 7.4 µs (IQR 1.2) | 197.8 µs (IQR 1.9) | 36.5 µs (IQR 1.8) | 184.1 µs (IQR 13.9) | 279.8 µs (IQR 90.4) |
| short-sparse-changing | exact_shared | 0.7 µs (IQR 0.0) | 9.6 µs (IQR 0.5) | 203.3 µs (IQR 15.8) | 61.0 µs (IQR 8.3) | 250.1 µs (IQR 14.4) | 197.7 µs (IQR 81.2) |
| long-dense-unchanged | binary64 | 2.0 µs (IQR 0.4) | 800.6 µs (IQR 64.4) | 2.48 ms (IQR 0.09) | 1.25 ms (IQR 0.12) | 2.33 ms (IQR 0.23) | 5.84 ms (IQR 0.29) |
| long-dense-unchanged | exact_shared | 1.8 µs (IQR 0.2) | 952.2 µs (IQR 163.4) | 2.97 ms (IQR 0.08) | 1.86 ms (IQR 0.35) | 2.48 ms (IQR 0.14) | 6.38 ms (IQR 0.39) |
| long-dense-changing | binary64 | 1.9 µs (IQR 0.1) | 477.8 µs (IQR 76.9) | 2.69 ms (IQR 0.15) | 1.06 ms (IQR 0.33) | 2.19 ms (IQR 0.04) | 5.47 ms (IQR 0.06) |
| long-dense-changing | exact_shared | 2.3 µs (IQR 0.3) | 1.27 ms (IQR 0.12) | 3.19 ms (IQR 0.12) | 1.42 ms (IQR 0.07) | 2.26 ms (IQR 0.09) | 6.32 ms (IQR 0.40) |
| long-sparse-unchanged | binary64 | 1.9 µs (IQR 0.2) | 88.6 µs (IQR 24.6) | 2.33 ms (IQR 0.24) | 402.2 µs (IQR 40.1) | 2.12 ms (IQR 0.14) | 1.78 ms (IQR 0.25) |
| long-sparse-unchanged | exact_shared | 1.9 µs (IQR 0.2) | 99.8 µs (IQR 5.0) | 2.36 ms (IQR 0.03) | 646.3 µs (IQR 6.5) | 2.28 ms (IQR 0.17) | 2.00 ms (IQR 0.25) |
| long-sparse-changing | binary64 | 1.9 µs (IQR 0.2) | 125.7 µs (IQR 30.1) | 2.27 ms (IQR 0.18) | 483.9 µs (IQR 68.6) | 2.29 ms (IQR 0.25) | 2.40 ms (IQR 0.23) |
| long-sparse-changing | exact_shared | 1.9 µs (IQR 0.1) | 156.1 µs (IQR 27.5) | 2.35 ms (IQR 0.13) | 844.3 µs (IQR 124.1) | 2.32 ms (IQR 0.09) | 2.44 ms (IQR 0.14) |

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
