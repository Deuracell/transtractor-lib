# The Transtractor

![PyPI version](https://img.shields.io/pypi/v/transtractor)
![Development Status](https://img.shields.io/pypi/status/transtractor)
![Tests](https://github.com/transtractor/transtractor-lib/actions/workflows/tests.yml/badge.svg)
![codecov](https://codecov.io/gh/transtractor/transtractor-lib/branch/main/graph/badge.svg)
![License](https://img.shields.io/github/license/transtractor/transtractor-lib)

## Universal PDF bank statement parsing

The Transaction Extractor, or 'Transtractor', is a universal library for extracting
transaction data from PDF bank statements. Available for both **Python** and **Rust**.

### Key Features

* **Fast**: Written in Rust, compiled to native code
* **Language-agnostic**: Use from Python or Rust without compromises
* **Lightweight**: Rules-based extraction (no AI/ML overhead)
* **Predictable**: 100% deterministic, audit-friendly extraction
* **Modular architecture**: Use just the parsing core, or the complete end-to-end API


## Architecture

Transtractor is organized as a **three-crate Rust workspace**:

### 1. **transtractor_core**
The pure parsing engine with zero external dependencies (except standard Rust libs):
- All statement parsing logic
- Format parsers (dates, amounts)
- Configuration management
- Statement validation & fixing
- Can be used in other Rust projects without Python or PDF overhead

### 2. **transtractor_py** (Python extension)
High-level Python API built on `transtractor_core`:
- Uses `pdfplumber` for PDF text extraction
- Provides user-friendly Python classes
- Drop-in replacement for PDFs → transactions

### 3. **transtractor_rs** (Rust API)
Complete Rust API equivalent to the Python version:
- Uses `pdfium-render` for PDF text extraction
- High-level `Parser` struct for end-to-end parsing
- Native Rust solution with no Python dependencies

## Installation

### Python Users: Install from PyPI

```bash
pip install transtractor
```

**Requirements**: Python 3.9 or higher

### Rust Users: Add to Cargo.toml

For the complete end-to-end API with PDF support:
```toml
[dependencies]
transtractor_rs = "0.9"
```

For just the parsing core (bring your own PDF extraction):
```toml
[dependencies]
transtractor_core = "0.9"
```

### Compile from Source

1. **Install Rust**: Download and install Rust from [rustup.rs](https://rustup.rs/)

2. **Install Maturin** (for Python): Install the build tool
   ```bash
   pip install maturin
   ```

3. **Build and install**:
   ```bash
   git clone https://github.com/transtractor/transtractor-lib.git
   cd transtractor-lib

   # For Python
   maturin develop --release

   # For Rust, just add to your Cargo.toml
   ```

## Usage

### Python Example

```python
from transtractor import Parser

# Create parser
parser = Parser()

# Load configuration(s)
parser.import_config_from_file('au_cba_debit.json')

# Parse PDF
statement = parser.parse_pdf('statement.pdf')

# Access results
print(f"Account: {statement.account_number}")
print(f"Transactions: {len(statement.transactions)}")

# Export to CSV
statement.to_csv('statement.csv')
```

### Rust Example

```rust
use transtractor_rs::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parser = Parser::new();

    // Load configuration
    parser.import_config_from_file("au_cba_debit.json")?;

    // Parse PDF
    let statement = parser.parse_pdf("statement.pdf", None)?;

    // Access results
    println!("Account: {}", statement.account_number.unwrap());
    println!("Transactions: {}", statement.proto_transactions.len());

    Ok(())
}
```

### Using transtractor_core (Rust, no PDF handling)

If you're handling PDF extraction yourself:

```rust
use transtractor_core::configs::db::ConfigDB;
use transtractor_core::parsers::flows::text_items_to_statement_datas;
use transtractor_core::structs::TextItem;

// You provide the TextItems (extracted from PDF however you like)
let text_items = vec![
    TextItem::new("Account".to_string(), 100, 50, 150, 60, 0),
    // ... more items ...
];

let mut db = ConfigDB::new(true, false);
db.register_from_file("config.json")?;
let config = db.get_config("my_bank")?;

let results = text_items_to_statement_datas(&text_items, &[config])?;
```

## Why Three Crates?

**Separation of concerns**:
- `transtractor_core` is pure parsing logic—no dependencies, fast compilation, easy to test
- `transtractor_py` adds Python-specific conveniences (pdfplumber, high-level types)
- `transtractor_rs` adds Rust-specific conveniences (pdfium-render, Rust idioms)

**Benefits**:
- Rust developers can use `transtractor_core` in embedded/headless scenarios without pulling in PDF libraries
- Python users get a seamless experience identical to other Python packages
- Rust users get a first-class API, not a second-class wrapper
- Both APIs benefit from the same battle-tested parsing logic
- Easy to add future language bindings (C#, Go, etc.) without modifying the core

## Advanced Usage

See the [documentation](https://transtractor-lib.readthedocs.io/en/latest/) for:
- Detailed API reference
- Supported statement types
- Custom configuration creation
- Troubleshooting & debugging

## Supported Statements

See the [documentation](https://transtractor-lib.readthedocs.io/en/latest/supported_statements.html) for a list of supported banks and statement types.

### Custom Configurations

You can create your own parsing configuration files. See the [configuration guide](https://transtractor-lib.readthedocs.io/en/latest/configuration.html):

```python
# Python
parser = Parser()
parser.import_config_from_file('my_custom_config.json')
statement = parser.parse_pdf('statement.pdf')
```

```rust
// Rust
parser.import_config_from_file('my_custom_config.json')?;
let statement = parser.parse_pdf('statement.pdf', None)?;
```

## Contributing

Contributions are welcome! New and well-tested configuration files are especially valuable.

- **Configuration files**: Submit to `python/transtractor/configs/` directory
- **Bug reports & features**: Open an issue on GitHub
- **Questions**: Email develop@transtractor.net

## License

MIT License - see LICENSE file for details
