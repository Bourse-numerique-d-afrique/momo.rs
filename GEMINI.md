# Gemini Code Understanding

## Project Overview

This project, `momo.rs`, is a Rust library for interacting with the MTN Mobile Money (MoMo) API. It provides a convenient and idiomatic Rust interface for the following MTN MoMo products:

*   **Collections:** Requesting payments from customers.
*   **Disbursements:** Sending money to customers.
*   **Remittances:** Cross-border money transfers.

The library supports both the sandbox and production environments. It also includes an optional callback server for handling webhook notifications from the MoMo API.

### Key Technologies

*   **Rust:** The project is written entirely in Rust.
*   **Tokio:** The library uses the `tokio` runtime for asynchronous operations.
*   **Reqwest:** The `reqwest` library is used for making HTTP requests to the MoMo API.
*   **Poem:** The optional callback server is built using the `poem` web framework.
*   **Serde:** The `serde` library is used for serializing and deserializing JSON data.

### Architecture

The library is structured into several modules:

*   `src/lib.rs`: The main entry point of the library, which defines the `Momo` struct and its methods.
*   `src/products/`: This module contains the implementation for the different MoMo API products (collections, disbursements, and remittances).
*   `src/requests/` and `src/responses/`: These modules define the data structures for the API requests and responses.
*   `src/enums/`: This module defines the various enums used in the library, such as `Currency` and `PartyIdType`.
*   `src/common/`: This module contains common utilities, such as the HTTP client and token manager.
*   `momo-callback-server/`: This directory contains a separate crate for the callback server.

## Building and Running

### Building the Library

To build the library, you can use the following command:

```bash
cargo build
```

### Running Tests

To run the tests, you can use the following command:

```bash
cargo test
```

### Running the Callback Server

To run the callback server, you can use the following command:

```bash
cargo run --package momo-callback-server
```

## Development Conventions

### Coding Style

The code follows the standard Rust coding style. The `rustfmt` tool is likely used to format the code.

### Testing

The project includes a suite of tests in the `tests/` directory. The tests use the `tokio` runtime and the `test-case` crate for writing table-driven tests. The tests also use environment variables to configure the MTN MoMo API credentials.

### Contributions

The project is hosted on GitHub and accepts contributions. The `README.md` file provides instructions on how to install and use the library.
