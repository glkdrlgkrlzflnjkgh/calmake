## CALMAKE REPO - CONTRIBUTION GUIDELINES

## PRELUDE - DEFINITIONS
*Worker Code* - code that runs off of the main thread in parallel to it.

## RESPECT
The calmake repository needs to be a safe space for all contributors, and as such,
the following behaviors are prohibited.
1. Harassment
2. Bullying
3. Discrimination
4. Being generally disrespectful to other contributors on the repository.
NB! This list is non-exhaustive! Use common sense! If you wouldn't do it in real life, don't do it here!
## NO TOLERANCE
Like most- if not all real-world public spaces, the Calmake Repository has no tolerance to abuse, <br>
and as such, abusive behavior toward contributors and/or maintainers is *STRICTLY* prohibited.
## POLICY ON AI PRs
As calmake is a build system, stability is vital, and because a lot of AI generated code is.... garbage....
(and also because AI companies are sending out their dumb little scrapers and using OSS work without consent)
AI generated PRs are prohibited.
## ON THE USAGE OF THE PANIC!() MACRO
Calmake runs builds in parallel via a system of workers. <br>
Because of this, a panic!() inside of *Worker Code* does **NOT** behave like a normal error- it will *poison* the shared state, <br>
which could cause serious bugs.<br> Even though the code explicitly guards against panics in a worker thread, do **NOT** take that protection for granted in your PR! <br>
It is required that you use anyhow::bail!() or return Err() or the ? operator instead of panic!() in *Worker Code!*

## SCHEDULER INVARIANTS
Calmake's scheduler relies on strict ordering and some shared state, <br>
and as such has several invariants. <br>
1. A target must NOT run until its dependencies are marked "Done"
2. A worker thread must NOT panic and should handle errors gracefully, while still reporting that something went wrong.
3. The DAG must NOT be mutated after it is computed, as if it does many classes of bugs could arise.
<br>
If code in a PR is found to violate one or more of these invariants, it will not be merged until they are corrected.