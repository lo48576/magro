# Rationale

## Listing `.git` by default (rather than working directory)

This is for consistency with bare repositories.
For use cases such as archiving, users might not want to checkout the repo contents
since creating working directory unnecessarily increases storage space usage.
Defaulting to listing working directories means that bare repositories are omittied by default,
but this is not desirable since bare repositories are "first class citizen" in magro.
