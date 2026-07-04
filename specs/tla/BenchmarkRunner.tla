---- MODULE BenchmarkRunner ----
EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS
    Instances,           \* Set of benchmark instance names
    MaxTime,             \* Maximum timeout per instance
    Workers              \* Number of parallel workers

VARIABLES
    state,               \* Current workflow state
    completed,           \* Set of completed instances
    failed,              \* Set of failed instances
    current_time         \* Elapsed time tracker

TypeInvariant ==
    /\ state \in {"INIT", "VALIDATE", "RUN", "SCORE", "REGRESS", "DONE", "ABORT"}
    /\ completed \subseteq Instances
    /\ failed \subseteq Instances
    /\ current_time \in Nat

Init ==
    /\ state = "INIT"
    /\ completed = {}
    /\ failed = {}
    /\ current_time = 0

Validate ==
    /\ state = "INIT"
    /\ state' = "VALIDATE"
    /\ UNCHANGED <<completed, failed, current_time>>

RunBenchmarks ==
    /\ state = "VALIDATE"
    /\ state' = "RUN"
    /\ UNCHANGED <<completed, failed, current_time>>

CompleteInstance(i) ==
    /\ state = "RUN"
    /\ i \in Instances \ (completed \union failed)
    /\ completed' = completed \union {i}
    /\ IF completed' = Instances
       THEN state' = "SCORE"
       ELSE state' = state
    /\ UNCHANGED <<failed, current_time>>

FailInstance(i) ==
    /\ state = "RUN"
    /\ i \in Instances \ (completed \union failed)
    /\ failed' = failed \union {i}
    /\ state' = "ABORT"
    /\ UNCHANGED <<completed, current_time>>

Score ==
    /\ state = "SCORE"
    /\ state' = "REGRESS"
    /\ UNCHANGED <<completed, failed, current_time>>

Regress ==
    /\ state = "REGRESS"
    /\ state' = "DONE"
    /\ UNCHANGED <<completed, failed, current_time>>

\* Liveness: Every started benchmark eventually completes or fails
Liveness == \A i \in Instances : 
    [](i \in completed \/ i \in failed)

\* Safety: Cannot abort and complete simultaneously
Safety == ~(state = "ABORT" /\ completed # {})

Next ==
    \/ Validate
    \/ RunBenchmarks
    \/ \E i \in Instances : CompleteInstance(i)
    \/ \E i \in Instances : FailInstance(i)
    \/ Score
    \/ Regress

Spec == Init /\ [][Next]_<<state, completed, failed, current_time>> /\ Liveness

====
