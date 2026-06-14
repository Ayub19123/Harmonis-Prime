# Reproduction Guide – Harmonis Prime (SET-5.6)

## One‑command validation

\\\ash
git clone https://github.com/Ayub19123/Harmonis-Prime.git
cd Harmonis-Prime
git checkout main
cargo test --all-targets --features pyo3 -- --nocapture
\\\

**Expected output:**  
\\\
106 tests passed, 0 failed, 0 ignored.
Zero warnings in sealed modules.
BRICK-51: 13/13 CMF certifications passed.
\\\

## Python bindings (optional)

\\\ash
python -m venv .venv
.venv\Scripts\Activate.ps1   # Windows
pip install maturin
maturin develop --features pyo3
python tests/py_bindings_test.py
\\\

## Hardware context

- Tested on: 11th Gen Intel i7-1165G7, 16GB RAM, Windows 11 / Ubuntu 22.04 (WSL2)
- No core pinning, no turbo locking – results may vary ±30%.
- For cycle‑accurate benchmarks, use \	askset\ and disable CPU frequency scaling.

Full whitepaper: [WHITEPAPER_HBS2_0.md](./WHITEPAPER_HBS2_0.md)
