#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int V = 6;

int main(int argc, char *argv[]) {
	int v = 88;
	pid_t pid;

	printf("before vfork\n");
	if (0 > (pid = vfork())) {
		perror("vfork");
		exit(1);
	} else if (0 == pid) {
		++V;
		++v;
		_exit(0); // child terminates
	}
	printf("pid = %ld, glob = %d, var = %d\n", (long)getpid(), V, v);
	exit(0);
}
