[![Build Status](https://github.com/linxli/ldns/actions/workflows/test-rust.yml/badge.svg)](https://github.com/linxli/ldns/actions)
[![Code Coverage](https://img.shields.io/badge/coverage-100%25-brightgreen)](https://github.com/linxli/ldns/actions)

# DNS
Learning how DNS Works and creating my own DNS service


# Usage
### Building and running the application

When you're ready, start the application by running:
`docker compose up --build`.

Your application will be available at http://localhost:53.

### Deploying the application to the cloud

First, build your image, e.g.: `docker build -t myapp .`.
If your cloud uses a different CPU architecture than your development
machine (e.g., you are on a Mac M1 and your cloud provider is amd64),
you'll want to build the image for that platform, e.g.:
`docker build --platform=linux/amd64 -t myapp .`.

Then, push it to your registry, e.g. `docker push myregistry.com/myapp`.

Consult Docker's [getting started](https://docs.docker.com/go/get-started-sharing/)
docs for more detail on building and pushing.

### References
* [Docker's Rust guide](https://docs.docker.com/language/rust/)
