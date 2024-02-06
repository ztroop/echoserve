# echoserve

A mock server tool designed for testing API requests. It allows you to specify endpoints and their responses through a YAML configuration file or defaults to a simple echo server if no file is provided.

## Features

- **Flexible Endpoint Configuration**: Define custom endpoints with associated JSON responses and status codes in a YAML file.
- **Default Echo Mode**: In the absence of a configuration file, EchoServe responds to all requests with a 200 OK status.
- **Simple and Lightweight**: Easy to set up and use for quick API testing.

## Usage

```sh
echoserve -p <PORT> [-c <PATH_TO_CONFIG_FILE>]

-p, --port: The port number EchoServe will listen on.
-c, --config: (Optional) Path to the YAML configuration file.
````
