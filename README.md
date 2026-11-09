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
- Dockerfiles for easier deployment (Early 2025)
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
