#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>

int main(void) {
	pid_t pid;

	if (0 > (pid = fork())) {
		perror("fork");
		exit(1);
	} else if (0 == pid) {
		if (0 > (pid = fork())) {
			perror("fork");
			exit(1);
		} else if (pid > 0)
			exit(0); // parent from second fork == first child

		// here we're the second child; our parent becomes init as
		// soon as our real parent calls exit() in the statement above.
		sleep(2);
		printf("second child, parent pid = %ld\n", (long)getppid());
		exit(0);
	}

	if (pid != waitpid(pid, NULL, 0)) {
		perror("waitpid");
		exit(1);
	}

	// the original process, not the parent of the second child
	exit(0);
}
