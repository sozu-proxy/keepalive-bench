# Benchmark tool for runtime reconfiguration of reverse proxies

This tool is used to test the behaviour of proxies while their configuration
is changed at runtime (whether they start new processes or update their routing
configuration in memory).

It uses a backend nodeJS application that replies with some metadata about the
HTTP communication, and a few HTTP clients that will call the proxy over and over.

It logs data to stdout:

- timestamp of the data (in μs since starting the test)
- which client made the call (identified by a number)
- the status code (if applicable) or a connection error message
- the TCP source port for the client<->proxy communication (to see if the front connection was closed and restarted)
- which backend instance answered to the request (identified by a number)
- the TCP source port for the proxy<->instance communication (to see if the backend connection was closed and restarted)

This data is also written to the `data.csv` file in the working directory.

You can also see that the last line of the terminal displays aggregated information:

- number of successful requests
- number of failed requests (TCP error, or status code different from 200)
- front changed (how many frontend TCP connections changed their source port)
- back changed (how many backend TCP connections changed their source port)

## Testing with sozu

starting the backends:
- node assets/server.js 0 1030 &
- node assets/server.js 1 1031 &
- node assets/server.js 2 1032 &
- node assets/server.js 3 1033 &
- sozu start -c config.toml
- sozu is now started with the two first servers
- cargo run http://lolcatho.st:8080/
- sozuctl -c config.toml backend add --id MyApp --ip 127.0.0.1 --port 1032
- we now added a backend server
- sozuctl -c config.toml backend remove --id MyApp --ip 127.0.0.1 --port 1030
- we now removed a backend server
