# echoserve

[![Build](https://github.com/ztroop/echoserve/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/ztroop/echoserve/actions/workflows/build.yml)

Echoserve is a mock API server that serves configurable responses based on a YAML configuration file. It supports static responses as well as response sequences for endpoints.

## Features
- Serve mock endpoints with configurable method, path, status, headers, and payload.
- Support for JSON, XML, HTML, and plain text responses.
- Sequence feature: endpoints can return a series of different responses in order.
- Simulated latency.

## Usage

```sh
cargo run -- --config examples/config.yml
```

### Command-line Options
- `-c, --config <FILE>`: Path to the YAML configuration file.
- `-p, --port <PORT>`: Port to listen on (default: 8080).
- `-a, --address <ADDR>`: Address to bind to (default: 127.0.0.1).
- `-l, --latency <MS>`: Simulated latency in milliseconds (default: 0).

## Configuration File Format

The configuration file is a YAML file containing a list of endpoint definitions. Each endpoint can specify a static response or a sequence of responses.

### Static Response Example
```yaml
- name: "Example Endpoint 1"
  endpoint: "/example1"
  method: "GET"
  data:
    format: "xml"
    payload: |
      <example>
        <id>12345</id>
        <description>Data for the first example</description>
      </example>
  status: 200
  headers:
    Custom-Header: "Value1"
```

### Sequence Response Example
```yaml
- name: "Sequence Example"
  endpoint: "/sequence"
  method: "GET"
  sequence:
    - data:
        format: "json"
        payload:
          message: "First response"
      status: 200
    - data:
        format: "json"
        payload:
          message: "Second response"
      status: 200
    - data:
        format: "json"
        payload:
          message: "Final response"
      status: 404
```

- On each request to `/sequence`, the next response in the sequence is returned. After the last response, the final response is repeated for subsequent requests.

### Not Found Example
```yaml
- name: "Not Found Example"
  endpoint: "/notfound"
  status: 404
```