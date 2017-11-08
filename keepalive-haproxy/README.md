# Install haproxy 

git clone http://git.haproxy.org/git/haproxy.git/
git checkout v1.8-rc2

``` bash
make clean
make TARGET=linux26 USE_GETADDRINFO=1 USE_ZLIB=1 USE_OPENSSL=1
```

# Run the bench for haproxy

Go in the assets directory and launch the server

`./assets/start-backends.sh`

Go in your haproxy folder where you have your bin compiled and your config file

`./haproxy -f config_haproxy.cfg -p ./haproxy.pid`

To reload haproxy with the latest release you only need to update the your config (config_haproxy.cfg) 
and launch an new haproxy. I'll alone kill the latest and close his current connections progessively:

`./haproxy -f config_haproxy.cfg -p haproxy.pid -st $(<./haproxy.pid)`

To make modifications juste edit your config file with `vim`, `gedit`, ...

DISCLAIMER: we use rsyslog to get all the logs print by haproxy

Run the bench client with:

`run cargo http://localhost:8080`
