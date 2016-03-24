ifeq ($(uname -s),FreeBSD)
	CC	:= clang
else
	CC	:= gcc
endif
CFLAGS	:= -g
LDFLAGS	:= -pthread

all: status clone

clone: clone.c
	cc $< -o $@
