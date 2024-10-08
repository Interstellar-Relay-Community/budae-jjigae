# [Budae Jjigae](https://en.wikipedia.org/wiki/Budae-jjigae) - ActivityPub Message Rejection Facility in Rust

Simpson Tang Edition - WORK IN PROGRESS

## License
AGPL-3.0

## How it works

This project quickly inspect `.object.content` key from ActivityStrema object poured into `/inbox`

and use Regex (currently hardcoded) to filter out potential spam.

## How to use

Detailed information will not be provided because this document cannot cover all setup.

Use following snipplets to adapt this for your own needs.

nginx.conf (for Mastodon, Misskey, etc)
```
    location /inbox {
        try_files $uri @budae;
    }

    location ~ /users/(.*)/inbox {
        try_files $uri @budae;
    }

    location @budae {
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Proxy "";
        proxy_pass_header Server;
        # Avoid 504 HTTP Timeout Errors
        proxy_connect_timeout       605;
        proxy_send_timeout          605;
        proxy_read_timeout          605;
        send_timeout                605;
        keepalive_timeout           605;

        proxy_pass http://127.0.0.1:7000;
        proxy_buffering off;
        proxy_redirect off;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;

        tcp_nodelay on;
    }
```

nginx.conf (for Lemmy)
```
    location /inbox {
        try_files $uri @budae;
    }

    location ~ /u/(.*)/inbox {
        try_files $uri @budae;
    }
    
    location ~ /c/(.*)/inbox {
        try_files $uri @budae;
    }

    location @budae {
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header Proxy "";
        proxy_pass_header Server;
        # Avoid 504 HTTP Timeout Errors
        proxy_connect_timeout       605;
        proxy_send_timeout          605;
        proxy_read_timeout          605;
        send_timeout                605;
        keepalive_timeout           605;

        proxy_pass http://127.0.0.1:7000;
        proxy_buffering off;
        proxy_redirect off;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection $connection_upgrade;

        tcp_nodelay on;
    }
```

Caddyfile (for Pleroma/Akkoma)
```
    route /inbox {
        reverse_proxy budae:3000
    }

    route /relay/inbox {
        reverse_proxy budae:3000
    }

    route /users/*/inbox {
        reverse_proxy budae:3000
    }
```


docker-compose.yml (If you're using Lemmy, change "web:3000" to "lemmy:8536" and remove networks section.)
```
  budae:
    image: perillamint/budae-jjigae:latest
    restart: always
    command:
      - "/usr/bin/budae-jjigae"
      - "--backend"
      - "web:3000"
    networks:
      - external_network
      - internal_network
    environment:
      - RUST_LOG=info
    ports:
      - '127.0.0.1:7000:3000'
    depends_on:
      - web
```
