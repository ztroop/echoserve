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

## Configuration

Each item in the list (denoted by a dash -) represents a configuration for an API endpoint. Each endpoint configuration consists of several key-value pairs that define its properties. Here's what each key represents:

- `name` - A human-readable identifier for the endpoint. It's used for reference and doesn't affect the endpoint's functionality.
- `endpoint` - The URI path that the endpoint responds to. This is the part of the URL that follows your domain or base URL.
- `method` - (Optional) Specifies the HTTP method (e.g., GET, POST) the endpoint should respond to. It defines the type of operation you want to perform.
- `data` - (Optional) Contains the data that the endpoint will send back in the response. This section is structured as nested key-value pairs, representing the response body. It's typically used with methods like POST or PUT that involve sending data.
- `status` - The HTTP status code that the endpoint will return. It indicates the result of the request (e.g., 200 for success, 404 for not found).
- `headers` - (Optional) Specifies any additional HTTP headers that the response will include. Headers are often used for specifying the content type or for authentication.

### File Example

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
