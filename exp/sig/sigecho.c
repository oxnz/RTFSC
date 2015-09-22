/*
 * Filename:	sigecho.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-09-22 13:02:50 CST]
 * Last-update:	2015-09-22 13:02:50 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License: 
 * Copyright (c) 2013 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <signal.h>

int exitflag = -2;

static void
sig_handler(int signo)
{
	switch (signo) {
		case SIGINT:
			printf("int\n");
			break;
		case SIGTERM:
			printf("term\n");
			break;
		default:
			printf("unknown signal: %d\n", signo);
			break;
	}
	++exitflag;
}

int
main(int argc, char *argv[])
{
	struct sigaction sigact;
	sigact.sa_handler = sig_handler;
	sigemptyset(&sigact.sa_mask);
	sigact.sa_flags = 0;
	if (sigaction(SIGINT, &sigact, NULL) == -1)
		err(1, "signal");
	if (sigaction(SIGTERM, &sigact, NULL) == -1)
		err(1, "signal");

	while (1) {
		pause();
		if (exitflag > 0)
			exit(0);
	}

	exit(0);
}
