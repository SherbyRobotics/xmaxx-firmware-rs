#!/bin/sh
#
# Install prerequesites on Debian based systems.
#
sudo apt install	\
	avr-libc	\
	gcc-avr		\
	pkg-config	\
	avrdude		\
	libudev-dev	\
	build-essential

cargo +stable install ravedude
