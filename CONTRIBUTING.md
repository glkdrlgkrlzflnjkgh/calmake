## CALMAKE REPO - CONTRIBUTION GUIDELINES

## PRELUDE - DEFINITIONS
*Worker Code* - code that runs off of the main thread in parallel to it.

## RESPECT
The calmake repository needs to be a safe space for all contributors, and as such,
the following behaviours are prohibited.
1. Harassment
2. Bullying
3. Discrimination
4. Being generally disrespectful to other contributors on the repository.
## POLICY ON AI PRs
As calmake is a build system, stability is vital, and because ALOT of AI generated code is.... garbage....
(and also because AI companies are sending out their dumb little scrapers and using FOSS work without consent)
AI generated PRs are prohibited.
## ON THE USAGE OF THE PANIC!() MACRO
Calmake runs builds in parallel via a system of workers. <br>
Because of this, a panic!() inside of *Worker Code* does **NOT** behave like a normal error- it will *poison* the shared state, <br>
which could cause serious bugs.<br> Even though the code explicitly gaurds against panics in a worker thread, do **NOT** take that protection for granted in your PR! <br>
It is required that you use anyhow::bail!() or return Err() or the ? operator instead of panic!() in *Worker Code!*
