\# Reproduction Log — Harmonis Prime



\## Witness #1 — Abdulwahab72



| Attribute | Value |

|-----------|-------|

| \*\*Witness\*\* | @abdulwahab72 |

| \*\*Date\*\* | 2026-06-09 |

| \*\*Machine\*\* | Windows 10 Pro |

| \*\*CPU\*\* | Intel Core i7-8565U |

| \*\*RAM\*\* | Not specified |

| \*\*OS\*\* | Windows 10 Pro |

| \*\*Rust Version\*\* | 1.96.0 |

| \*\*Tag Tested\*\* | v7.1.1-SPEC-HBS1.1 |

| \*\*Commit\*\* | 14f1de0 |



\### Commands Executed

```powershell

git clone https://github.com/Ayub19123/Harmonis-Prime.git

cd Harmonis-Prime

git checkout v7.1.1-SPEC-HBS1.1

cargo test

cargo audit

cargo run --release --bin benchmark -- 10000 0x51C3\_2026\_0613

