#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>

void print_exstat(int status) {
	if (WIFEXITED(status))
		printf("normal termination, exit status = %d\n",
				WEXITSTATUS(status));
	else if (WIFSIGNALED(status))
		printf("abnormal termination, signal number = %d%s\n",
				WTERMSIG(status),
#ifdef WCOREDUMP
				WCOREDUMP(status) ? " (core dumped)" : "");
#else
	"");
#endif
	else if (WIFSTOPPED(status))
		printf("child stopped, signal number = %d\n",
				WSTOPSIG(status));
}

int main(int argc, char *argv[]) {
	pid_t pid;
	int status;

	if (0 > (pid = fork())) {
		perror("fork");
		exit(1);
	} else if (0 == pid) {
		exit(7);
	}

	if (pid != wait(&status)) {
		perror("wait");
		exit(1);
	}
	print_exstat(status);

	if (0 > (pid = fork())) {
		perror("fork");
		exit(1);
	} else if (0 == pid)
		abort();

	if (pid != wait(&status)) {
		perror("wait");
		exit(1);
	}
	print_exstat(status);

	if (0 > (pid = fork())) {
		perror("fork");
		exit(1);
	} else if (0 == pid)
		status /= 0;

	if (pid != wait(&status)) {
		perror("wait");
		exit(1);
	}
	print_exstat(status);

	exit(0);
}
