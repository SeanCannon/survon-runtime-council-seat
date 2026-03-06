# Survon Council Seat

A configurable council member seat that provides domain-specific expertise to the Survon smart homestead system. Each seat runs as an independent instance with a specific strategy (domain expertise).

## Strategies

| Strategy | Description |
|----------|-------------|
| `librarian` | Reads and searches static knowledge from documents/manifests |
| `medicine` | Medical expertise (requires specialized LLM) |
| `mechanical` | Mechanical engineering expertise |
| `botany` | Agriculture and plant expertise |
| `veterinary` | Animal health expertise |
| `building` | Construction and architecture expertise |
| `survival` | Survival skills and preparedness |

## Installation

### Quick Install (Raspberry Pi)

```bash
curl -sSL https://raw.githubusercontent.com/survon/survon-council-seat/master/scripts/install.sh | bash -s -- --strategy librarian
```

### Manual Install

1. Download the latest binary:
```bash
curl -L https://github.com/survon/survon-council-seat/releases/latest/download/survon-council-seat -o /usr/local/bin/survon-council-seat
chmod +x /usr/local/bin/survon-council-seat
```

2. Set environment variables:
```bash
export COUNCIL_STRATEGY=librarian
export DATABASE_PATH=./data/council.db
export LOG_LEVEL=info
```

3. Run:
```bash
./survon-council-seat
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `COUNCIL_STRATEGY` | Strategy name (librarian, medicine, etc.) | `librarian` |
| `DATABASE_PATH` | Path to knowledge database | `./data/council.db` |
| `LLM_ENDPOINT` | Optional LLM API endpoint | None |
| `LLM_API_KEY` | Optional LLM API key | None |
| `LOG_LEVEL` | Logging level (debug, info, warn, error) | `info` |

### The Librarian Strategy

The librarian is the default strategy that reads from static knowledge. It uses SQLite FTS5 for fast full-text search.

**Loading Knowledge:**

Knowledge is stored in the database and can be searched. The librarian automatically loads sample data on first run.

**Query Format:**

Simply ask questions and the librarian will search its knowledge base and return relevant excerpts with source citations.

## Running Multiple Seats

To run multiple council members (e.g., on different Raspberry Pis):

**Pi 1:**
```bash
export COUNCIL_STRATEGY=librarian
./survon-council-seat
```

**Pi 2:**
```bash
export COUNCIL_STRATEGY=medicine
./survon-council-seat
```

**Pi 3:**
```bash
export COUNCIL_STRATEGY=botany
./survon-council-seat
```

## Survon OS Integration

The council seat can be installed via Survon OS menu:

1. Select "Install Council Seat"
2. Choose strategy
3. Configure optional LLM endpoint
4. Launch

## Development

```bash
# Build
cargo build --release

# Run with custom strategy
COUNCIL_STRATEGY=librarian cargo run --release

# Test
cargo test
```

## License

MIT License
