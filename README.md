# Newsletter Application

## Database

Datastore: `cockroach`

Note: see Makefile for how to bring the datastore up or down

### Migrations

#### Pre-requisites

- Cargo bininstall
- Cargo

Install diesel

#### Creating Migration Files

```zsh
diesel migration --migration-dir ./db/migrations generate create_subscriptions
```

### Database connectivity

Connection string for local development:

```bash
jdbc:postgresql://localhost:26257/newsletter?sslmode=disable&user=root
```    
