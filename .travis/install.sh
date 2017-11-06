#!/bin/bash
set -eu

case "$TRAVIS_OS_NAME" in
    linux)
        sudo apt-get update
        sudo apt-get install -y capnproto
        ;;
    mac)
        brew update
        brew install capnp
        ;;
    *)
        echo 'Unknown OS!'
        exit 1
        ;;
esac
