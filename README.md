# iSHARE Authorization Registry

An open source implementation of an iSHARE-compliant Authorization Registry. 

## Overview

The iSHARE Authorization Registry provides a complete solution for managing delegation and authorization information within the iSHARE framework. It features a RESTful backend API and a web-based management interface. The system currently handles user management through an iSHARE-enabled Identity Provider (IDP), such as our recommended implementation at https://github.com/LIFE-Electronic/keycloak-ishare.
## Key Features

The Authorization Registry supports core iSHARE functionality including:

- Storage and management of access policies
- Fully-featured REST API
- Management UI (optional)
- Verification of machine-to-machine (M2M) authorization requests
- Delegation evidence issuance and validation
- Compliance with iSHARE's trust framework

## Roadmap

- Fully pass iSHARE conformance tests (Early 2025)
- IDP independence to support more IDP providers than LIFE's Keycloak fork (sponsors needed)


## System requirements

- Rust (latest stable version)
- Node.js (LTS version)
- PostgreSQL (versions 12.20 or 14.15 verified)

### Additional requirements

- Valid iSHARE certificates for testing/deployment (obtainable from: https://ca7.isharetest.net:8442/ejbca/ra/index.xhtml)
- Registered, active participant in iSHARE network
- iSHARE Certificate chain (available at https://ca7.isharetest.net:8442/ejbca/retrieve/ca_crls.jsp. Select "TEST iSHARE EU Issuing Certification Authority G5" in PEM format)
- iSHARE-compatible Keycloak implementation as IDP (such as: https://github.com/LIFE-Electronic/keycloak-ishare)
- Administrator account in the iSHARE-enabled realm with `dexspace_admin` privileges

## Installation Guide

### Backend Setup

This section explains how to setup the AR in development mode. 

To make use of all features that the AR provides, you need to build and run the backend as well as the frontend. 

1. Clone repository

```bash
git clone https://github.com/LIFE-electronic/authorization-registry
cd authorization-registry
```

2. Build the backend

```bash
cd authorization-registry
cargo build
```

3. Configure PostgreSQL

```sql
postgres=# create database ar;
postgres=# create role ar with login;
postgres=# \password ar
postgres=# grant all privileges on database ar to ar;
```
4. Initialize the database

```bash
cargo install sea-orm-cli
export PASSWORD=
export DATABASE_URL="postgres://ar:$PASSWORD@localhost:5432/ar"
sea-orm-cli migrate up
```

5. Create `.config.json`

```
{
  "database_url": "$DATABASE_URL",
  "client_eori": "$CLIENT_EORI",
  "client_cert_path": "$CLIENT_CERT",
  "client_cert_pass": "$CLIENT_PASS",
  "jwt_secret": "$JWT_SECRET",
  "satellite_url": "$SATELLITE_URL",
  "satellite_eori": "$SATELLITE_EORI",
  "ishare_ca_path": "$ISHARE_CERTIFICATE_CHAIN",
  "idp_url": "$IDP_URL",
  "idp_eori": "$IDP_EORI",
  "log": { "human_readable": true },
  "deploy_route": "",
  "frontend": "$FRONTEND" 
}
```

Here's a comprehensive table outlining each variable, its description, and an example value:

| Variable Name | Description | Example |
|--------------|-------------|----------|
| $DATABASE_URL | The connection string for the database | `postgresql://ar:password@localhost:5432/ar` |
| $CLIENT_EORI | EORI number of the Authorization Registry | `EU.EORI.NL000000002` |
| $CLIENT_CERT | File path to the client's iSHARE certificate in p12 format | `/etc/ishare/certs/client.p12` |
| $CLIENT_PASS | Password for the client certificate | `certpass123` |
| $JWT_SECRET | Secret key for JWT token generation and validation | `your-secure-secret-key-here` |
| $SATELLITE_URL | URL of the iSHARE Satellite middleware service | `https://satellite-mw.isharetest.net` |
| $SATELLITE_EORI | EORI number of the iSHARE Satellite | `EU.EORI.NL000000001` |
| $ISHARE_CERTIFICATE_CHAIN | File path to the iSHARE certificate chain in pem format | `/etc/ishare/certs/chain.pem` |
| $IDP_URL | URL of the Identity Provider service | `https://idp.isharetest.net` |
| $IDP_EORI | EORI number of the Identity Provider | `EU.EORI.NL000000003` |
| $FRONTEND | Frontend configuration to configure parts of the frontend: e.g. the footer. | [see below](#frontend-example) |

#### $FRONTEND example
 ```json
 {
    "footer": {
      "navigation": {
        "passport": "http://passport.com",
        "catalogue": "http://catalogue.com",
        "authorization_registry": "http://ar.com",
        "datastation": "http://datastation.com"
      },
      "general": {
        "become_member": "http://become_member",
        "faq": "http://faq.com",
        "about": "http://about.com",
        "support": "http://support.com"
      },
      "contact": {
        "address": {
          "name": "Kantoor Utrecht",
          "address_content": [
            "Vleutensevaart 100",
            "3532 AD UTRECHT",
            "The Netherlands"
          ]
        },
        "tax_number": "76659119",
        "email": "info@dexes.nl",
        "phone_number": "+31 6 4637 4892"
      },
      "socials": {
        "linkedin": "http://linkedin.com",
        "x": "http://x.com"
      }
    }
  }
``` 

Note: When you run the ar with `cargo run`, put this directory into the `authorization-registry` directory (same where you run cargo build).

Note: The certificate chain file for the iSHARE test network can be downloaded here: https://ca7.isharetest.net:8442/ejbca/retrieve/ca_crls.jsp . Use the CA "TEST iSHARE EU Issuing Certification Authority G5" and pem format.

6. Optional: Run tests (requires superuser database privileges):

```sql
postgres=# alter user ar with superuser;
```

```bash
cargo test
```

7. Launch the backend

```
export RUST_LOG=tower_http=debug,authorization_registry=debug
cargo run
```

The backend will run on `http://localhost:4000`.

The REST API is documented at `http://localhost:4000/swagger-ui`.

The iSHARE capabilities are located at `http://localhost:4000/capabilities`.

## Frontend Setup

1. Install dependencies

```
cd ar-frontend
npm install
```

2. Configure the environment: Create .env in the ar-frontend directory:

```
VITE_BASE_API_URL=http://localhost:4000
```

Here's a comprehensive table outlining each variable, its description, and an example value:

| Variable Name | Description | Example |
|--------------|-------------|----------|
| $VITE_BASE_API_URL | The URL of the AR backend | `http://localhost:4000/api` |

3. Launch the frontend

```
npm start
```

## Documentation

For detailed information about the rest UI. Visit the route `swagger-ui` from the backend.

The implementation follows the official iSHARE specifications available at [dev.ishare.eu](https://dev.ishare.eu).

## Contributing

We welcome contributions to this project. Please read our contributing guidelines before submitting pull requests.

## License

This project is licensed under the GNU General Public License v3.0 - see the [LICENSE.md](LICENSE) file for details.

## Support

For questions and support:
- Open an issue in the GitHub repository
- Consult the documentation
- Contact the maintainers

## Security

Please report security vulnerabilities responsibly by contacting the maintainers directly. Do not create public issues for security concerns.

## Backend container

Build from the repository root:

```sh
docker build --pull -t authorization-registry:local .
docker run --rm --network none authorization-registry:local --help
```

The multi-stage image compiles the locked Cargo dependencies on Debian Bookworm
and runs the backend with OpenSSL and CA certificates on the same distribution.
It runs as UID/GID `10001:10001`. The frontend is built and hosted separately.
The build context excludes configuration, certificates, seeds, logs, and local
build artifacts. No database or iSHARE credentials are needed to build the image.

Build for the deployment server's architecture. On Apple Silicon, a local default
build is ARM64; for an x86 server use:

```sh
docker buildx build --platform linux/amd64 --load -t authorization-registry:local .
```

Emulated compilation can be slow.
The container CI workflow builds and smoke-tests AMD64 on GitHub-hosted runners.
Pull requests targeting `main` and manual runs only build and check. Pushes to
`main`, including PR merges, also publish the image directly to UpCloud.

### Registry setup

In the **AR repository's** Settings → Secrets and variables → Actions, configure:

| Type | Name | Value |
| --- | --- | --- |
| Variable | `REGISTRY_HOST` | Registry hostname, without `https://` or a path |
| Secret | `REGISTRY_USERNAME` | Dedicated registry user with Read + Create access |
| Secret | `REGISTRY_PASSWORD` | That registry user's password |

Leave the registry's public IP allowlist empty and keep anonymous pulls disabled.
Standard GitHub-hosted runners have changing outbound IPs; allowing only the AR
server or a former runner IP would block publishing. Authentication is still
required. See [UpCloud access control](https://upcloud.com/docs/products/container-registry/access-control/).

Checks can run without registry credentials; if the hostname is unset, they use a
local `localhost/ar` tag. Publishing fails explicitly if required settings are
missing. Credentials are supplied only to the publishing step, which never runs
for PRs or manual triggers.

Published images use `REGISTRY_HOST/ar:FULL_COMMIT_SHA`. No `latest` tag is updated.
Rebuilding a commit can produce a different image as base images and OS packages
change; Read + Create permissions prevent overwriting an existing tag with a
different image. Promote the existing image by digest after testing. Publishing
does not deploy the application.

### Configuration and startup

Create a directory on the server, for example `/etc/ar`, containing `config.json`
and the certificates required by the existing backend configuration. Use absolute
container paths such as `/etc/ar/client.p12` and `/etc/ar/ishare-chain.pem` in the
JSON. Make the directory traversable and its files readable by UID/GID 10001,
without making secrets world-readable (for example, directory `0750` and files
`0640`, owned by `root:10001`). Keep these files outside the repository.

Set `listen_address` to `0.0.0.0:4000`. The database URL should use the managed
PostgreSQL private hostname and port, with TLS verification, for example
`postgresql://USER:URL_ENCODED_PASSWORD@HOST:PORT/ar?sslmode=verify-full&sslrootcert=/etc/ar/postgres-ca.pem`.
Mount the database CA certificate alongside the other certificates. The server
must already have access to the database's private network.

```sh
export AR_CONFIG_DIR=/etc/ar
docker compose up -d --build
docker compose logs --tail=100 ar
```

Compose binds port 4000 only to host loopback for a host-based HTTPS reverse proxy.
For a basic HTTP check, run `curl --fail http://127.0.0.1:4000/swagger-ui/` on the
server (adjust for `deploy_route` if configured). This checks HTTP availability,
not the complete authorization flow. HTTPS routing and frontend deployment are
separate setup steps.

AR currently applies migrations automatically at startup, followed by seeds if
`seed_folder` is configured. The configured database role therefore needs schema
migration permissions. Start a single instance during deployment; back up and
review migrations before upgrades. Returning to an older image does not revert
the database schema. Omit `seed_folder` unless explicitly provisioning seed data;
if needed, mount the intended files and use their container path.

For a prebuilt image set `AR_IMAGE` to its immutable tag or digest and run
`docker compose pull` followed by `docker compose up -d --no-build`. First log in
on the server using a separate Read-only registry user. Publishing an image does
not deploy it; deployment remains manual.
