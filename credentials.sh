#!/bin/sh

# Generate 18 random alphanumeric characters base64 encode It
password=$(head -c 18 /dev/urandom | base64)

echo "Basic Authentication; user=machine password=$password"
echo $password > /tmp/machine_password

caddy_hash=$(caddy hash-password --plaintext $password)

echo "(machine_basic_auth) {
    basic_auth {
        machine $caddy_hash
    }
}" | sudo tee /var/lib/caddy/auth_snippet.conf

# Transfer ownership to caddy user
sudo chown caddy:caddy /var/lib/caddy/auth_snippet.conf

# Tigther permissions (Only readable by root or caddy user)
sudo chmod 600 /var/lib/caddy/auth_snippet.conf

sudo systemctl restart caddy.service

