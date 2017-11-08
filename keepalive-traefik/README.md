#Get traefik

Get the latest release at [traefik release](https://github.com/containous/traefik/releases)

#Run the test

`./traefik -c config_traefik.toml`

Go in the assets directory and launch the server

`./assets/start-backends.sh`

Return in the keepalive-bench folder and then run the benchmark client:

`cargo run http://localhost:8080`

Traefik watch the diff of `config_traefik.toml` to know when reload the proxy with the new configs file
So we provide a `config_traefik_update.toml` where the config derive than the `config_traefik.toml`

The first hot reload can be made by run the command: `cat `config_traefik_update.toml` > `config_traefik.toml`.
You can make other update by simply edit the config file with `vim`, `gedit`,... to launch a reload of the reverse proxy.
 
