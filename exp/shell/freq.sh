#!/usr/bin/env sh
#
# ===============================================================
#
# Filename:	freq.sh
#
# Author:		Oxnz
# Email:		yunxinyi@gmail.com
# Created:		[2015-09-14 21:15:23 CST]
# Last-update:	2015-09-14 21:15:23 CST
# Description: ANCHOR
#
# Version:		0.0.1
# Revision:	[None]
# Revision history:	[None]
# Date Author Remarks:	[None]
#
# License:
# Copyright (c) 2013 Oxnz
#
# Distributed under terms of the [LICENSE] license.
# [license]
#
# ===============================================================
#

if [ $# -lt 1 ]; then
	echo "usage: $0 file ..." >&2
	exit 1
fi

for file in "$@"; do
	if [ -f "$file" ]; then
		cat "$file" | xargs -n 1 | sort | uniq -c | sort -nr
	else
		echo "cannot open file: $file" >&2
	fi
done
