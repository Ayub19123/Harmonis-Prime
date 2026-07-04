# Baseline Comparison Framework — M2.7.16

## Purpose
Track Harmonis Prime performance against reference solvers (Kissat, CaDiCaL, Glucose).

## Methodology
1. Run identical benchmark suite on all solvers
2. Record PAR-2 scores per solver
3. Generate comparative cactus plots
4. Store results in this directory

## PAR-2 Formula
PAR-2 = (Σ solved_time + Σ unsolved(2 × timeout)) / N

## Competition Context
- SAT Competition 2027 uses PAR-2 as primary ranking metric
- Cactus plots required for artifact evaluation
- Baseline comparison demonstrates competitiveness
