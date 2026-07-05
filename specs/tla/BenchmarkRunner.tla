---- MODULE BenchmarkRunner ----
EXTENDS Naturals, FiniteSets

CONSTANTS Instances, MaxTime

VARIABLES state, completed, failed, scores, timestamp

States == {"INIT","VALIDATE","RUN","SCORE","REGRESS","DONE"}

TypeInvariant ==
    /\ state \in States
    /\ completed \subseteq Instances
    /\ failed \subseteq Instances
    /\ scores \in [Instances -> Nat]
    /\ timestamp \in Nat

Init ==
    /\ state = "INIT"
    /\ completed = {}
    /\ failed = {}
    /\ scores = [i \in Instances |-> 0]
    /\ timestamp = 0

Validate ==
    /\ state = "INIT"
    /\ state' = "VALIDATE"
    /\ timestamp' = timestamp + 1
    /\ UNCHANGED <<completed, failed, scores>>

RunBenchmarks ==
    /\ state = "VALIDATE"
    /\ state' = "RUN"
    /\ timestamp' = timestamp + 1
    /\ UNCHANGED <<completed, failed, scores>>

CompleteInstance(i) ==
    /\ state = "RUN"
    /\ i \in Instances \ (completed \cup failed)
    /\ scores' = [scores EXCEPT ![i] = @ + 1]
    /\ completed' = completed \cup {i}
    /\ timestamp' = timestamp + 1
    /\ UNCHANGED <<state, failed>>

FailInstance(i) ==
    /\ state = "RUN"
    /\ i \in Instances \ (completed \cup failed)
    /\ failed' = failed \cup {i}
    /\ scores' = [scores EXCEPT ![i] = MaxTime * 2]
    /\ timestamp' = timestamp + 1
    /\ UNCHANGED <<state, completed>>

Score ==
    /\ state = "RUN"
    /\ completed \cup failed = Instances
    /\ state' = "SCORE"
    /\ timestamp' = timestamp + 1
    /\ UNCHANGED <<completed, failed, scores>>

Regress ==
    /\ state = "SCORE"
    /\ state' = "REGRESS"
    /\ timestamp' = timestamp + 1
    /\ UNCHANGED <<completed, failed, scores>>

Done ==
    /\ state = "REGRESS"
    /\ state' = "DONE"
    /\ timestamp' = timestamp + 1
    /\ UNCHANGED <<completed, failed, scores>>

Next ==
    \/ Validate
    \/ RunBenchmarks
    \/ \E i \in Instances : CompleteInstance(i)
    \/ \E i \in Instances : FailInstance(i)
    \/ Score
    \/ Regress
    \/ Done

Safety ==
    /\ completed \cap failed = {}
    /\ timestamp <= Cardinality(Instances) + 6

Liveness == state = "INIT" ~> (state = "DONE" \/ state = "REGRESS")

Par2Bounded == \A i \in Instances : scores[i] <= MaxTime * 2

Spec == Init /\ [][Next]_<<state, completed, failed, scores, timestamp>> /\ WF_<<state, completed, failed, scores, timestamp>>(Next)

====
