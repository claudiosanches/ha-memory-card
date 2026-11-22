#!/bin/sh
# dehydrated hook: answers Let's Encrypt's DNS challenge with a TXT record on
# DuckDNS. DUCKDNS_TOKEN comes from the environment.
set -eu

case $1 in
deploy_challenge) query="txt=$4" ;;
clean_challenge) query="txt=removed&clear=true" ;;
*) exit 0 ;;
esac

# Through a config on stdin, so the token stays off the command line.
answer=$(printf 'url = "https://www.duckdns.org/update?domains=%s&token=%s&%s"\n' \
    "${2%.duckdns.org}" "$DUCKDNS_TOKEN" "$query" | curl -fsS -K -)
if [ "$answer" != OK ]; then
    echo "DuckDNS refused the challenge record" >&2
    exit 1
fi
