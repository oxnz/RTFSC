/*
 * Filename:	rollover.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-11-30 14:15:02 CST]
 * Last-update:	2015-11-30 14:15:02 CST
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

/*
 * this is used to test what will happen if the pid rollover
 */
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

// read from /proc/sys/kernel/pid_max
const int PID_MAX=131072;

int main(int argc, char *argv[]) {

	pid_t pid;
	while (pid = fork()) {
		if (pid == -1) {
			perror("fork");
			return 1;
		}
		usleep(10);
	}
	if (getpid() == 128811) {
		while (1) {
			printf("alive\n");
			sleep(1);
		}
	}
	return 0;
}
