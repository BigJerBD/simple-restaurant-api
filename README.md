simple-restaurant-api
===============
A simple restaurant API that allows for the creation of orders.

## Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) 
- [Python](https://www.python.org/downloads/) (for pre-commit hooks)
- [Docker+Docker-Compose](https://docs.docker.com/get-docker/) 

## Initial Setup

```bash
crago install sqlx-cli
copy .env.example .env

# Leave Database up for other commands
docker-compose up db --detach
sqlx migrate run  


cargo build
```

# Usage

## Swagger

```bash
# (Database Required)
cargo run
# -> go to http://localhost:8080/swagger-ui/index.html
```

##  Using prod Image
The prod image is mapped to port 8081 instead of 8080.

Use Docker compose to start the prod image as a container:

```bash 
# (Database Required)
cargo sqlx prepare -- --all-targets --all-features  

docker-compose up --build

```
## Testing with test client
To adjust settings simply edit the main.rs file in the client-test folder.
```bash
# Start the server from prod image or local build
cd client-test
cargo run
```


# Development

### Testing

```bash
# (Database Required)
cargo test
```

### Formatting

```bash
cargo fmt
cargo fix
cargo clippy
```

###  Installing Pre-commit Hooks

```bash
pip install pre-commit
pre-commit install
```

### Generating Migration

```bash
sqlx migrate add <migration-name>
```

# Solution Details

### Technical Decisions
- SQLX was opted for the solution as for personal curiosity and learning. ORMS such as
  Diesel could also have been an opted for easier query manipulation.
- The use of sequential ID for order  was made for simplicity but should not be used in a production situation.
  UUIDs could have been used  for uniqueness

### Omissions
- Pagination, Authentication,  and Deployment are omitted for simplicity.
- Unit testing is omitted since no apparent real business logic was seen in the assessment.
  However, Integration tests are present to test the API endpoints.
- Table Service is omitted for simplicity. Ideally, a table service could allow us to know what are the tables.





