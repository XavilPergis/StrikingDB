#!/bin/bash
set -eu

case "$TRAVIS_OS_NAME" in
	linux)
		curl -O https://capnproto.org/capnproto-c++-0.6.1.tar.gz
		tar zxf capnproto-c++-0.6.1.tar.gz
		cd capnproto-c++-0.6.1
		./configure
		make -j6 check
		sudo make install
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
