/*
 * Filename:	clone.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		[2015-10-05 18:01:55 CST]
 * Last-update:	2015-10-05 18:01:55 CST
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

#define _GNU_SOURCE
#include <sys/mman.h>
#include <sched.h>
#include <signal.h>
#include <sys/wait.h>
#include <unistd.h>
#include <sys/types.h>
#include <sys/syscall.h>
#include <err.h>
#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <stdlib.h>

#define STACK_SIZE (1 << 23) // 8M

int thread_func(void *param) {
	printf("------------------->\n");
	printf("thread id: %d\n", (int)syscall(SYS_gettid));
	printf("thread get param: %d\n", *(int *)&param);
	sleep(1);
	printf("<-------------------\n");

	return 0;
}

void child_handler(int signo) {
	printf("got signal: SIGCHLD\n");
}

int main(int argc, char *argv[]) {
	setvbuf(stdout, NULL, _IONBF, 0);
	signal(SIGCHLD, child_handler);

	printf("parent id: %d\n", getpid());
	void *pstack = (void *)mmap(NULL,
			STACK_SIZE,
			PROT_READ | PROT_WRITE,
			MAP_PRIVATE | MAP_ANONYMOUS | MAP_ANON, // | MAP_GROWSDOWN
			-1,
			0);
	if (MAP_FAILED == pstack)
		err(1, "mmap");
	printf("strace addr: %p\n", pstack);
	int ret = clone(thread_func,
			(void *)((unsigned char *)pstack + STACK_SIZE),
			CLONE_VM | CLONE_FS | CLONE_FILES | CLONE_SIGHAND | SIGCHLD,
			(void *)NULL);
	if (ret == -1)
		err(1, "clone");
	printf("start thread: %d\n", ret);
	sleep(5);
	pid_t pid = waitpid(-1, NULL, __WCLONE | __WALL);
	printf("child: %d exit\n", pid);
	exit(0);
}
