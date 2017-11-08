#!/bin/sh

#Download haproxy
git clone http://git.haproxy.org/git/haproxy.git/
git checkout v1.8-rc2

#compile haproxy with your OWN ARGS
make clean
make TARGET=linux26 USE_GETADDRINFO=1 USE_ZLIB=1 USE_OPENSSL=1

