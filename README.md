[![Build](https://github.com/ztroop/echoserve/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/ztroop/echoserve/actions/workflows/build.yml)

# echoserve

A mock server tool designed for testing API requests. It allows you to specify endpoints and their responses through a YAML configuration file or defaults to a simple echo server if no file is provided.

## Features

- **Flexible Endpoint Configuration**: Define custom endpoints with associated JSON responses and status codes in a YAML file.
- **Default Echo Mode**: In the absence of a configuration file, `echoserve` responds to all requests with a `200 OK` status.
- **Simple and Lightweight**: Easy to set up and use for quick API testing.

## Usage

```sh
Usage: echoserve [OPTIONS] -p <PORT>

Options:
  -p <PORT>         The port number to listen on.
  -a <ADDRESS>      (Optional) The address to listen on. Default: 127.0.0.1
  -c <CONFIG>       (Optional) Path to the YAML configuration file.
  -h, --help        Print help
  -V, --version     Print version
```

### Configuration File Example

```yaml
- name: "Example Endpoint 1"
  endpoint: "/example1"
  method: "POST"
  data:
    message: "This is the first example response."
    details:
      info: "More details about Example 1"
  status: 200
  headers:
    Content-Type: "application/json"
    Custom-Header: "Value1"

- name: "Example Endpoint 2"
  endpoint: "/example2"
  method: "POST"
  data:
    success: true
    payload:
      id: 12345
      description: "Data for the second example"
  status: 201

- name: "Not Found Example"
  endpoint: "/notfound"
  status: 404
```
