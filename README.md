# Python-Rust Apache Arrow interop example

This project contains an example Python module,
implemented in Rust using [pyo3](https://github.com/PyO3/pyo3).
The Python module accepts an [Apache Arrow](https://arrow.apache.org/) array and doubles the values in it.

It uses the Apache Arrow C Data Interface.

This code is mostly copied from the Apache Arrow [integration test](https://github.com/apache/arrow/commit/1d2b4a55770fa4dbe24959b3b40c745964c3184e#diff-c8cff1e3449398e7f468b57ceb3271f5dedf3c0c721661405b0d8ce25cb86f66) that introduced the interop, with some Clippy warnings fixed.

## Usage

Install Python poetry, e.g.:

```bash
pip install poetry
```

Install Python dependencies:

```bash
poetry install
```

Build the Rust Python module:

```bash
poetry run maturin develop
```

Run the example:

```bash
$ poetry run python run.py
[
  2,
  4,
  6
]
```

## Related reading

- [NiklasMolin/python-rust-arrow](https://github.com/NiklasMolin/python-rust-arrow) that explores the performance of IPC based interop.
