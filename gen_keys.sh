#!/bin/bash

openssl genpkey -algorithm RSA -out self_signed_certs/private_key.pem
openssl req -x509 -nodes -days 365 \
            -key self_signed_certs/private_key.pem \
            -out self_signed_certs/certificate.pem \
            -subj "/C=FR/ST=Paris/L=Paris/O=Test/OU=Test/CN=mydomain.com/emailAddress=test@example.com"

