A routing engine for public transit implementing Scalable transfer patterns.
Not (yet) meant to be a fully featured routing engine, but rather an exploration of the algorithms out there.

# Goals

- Performance & Efficiency
  - Preprocessing on Baden-Württemberg:
    - < 3h on laptop (all cores)
    - < 2GB peak RAM
    - < 5GB disk in addition to dataset
  - Queries on Baden-Württemberg:
    - < 50ms
    - < 500MB auxilary RAM per query (in addition to precomputed data)
    - no extra disk space