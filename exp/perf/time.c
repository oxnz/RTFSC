#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/time.h>
#include <sys/wait.h>
#include <sys/resource.h>

//	printf("%Uuser %Ssystem %Eelapsed %PCPU (%Xtext+%Ddata %Mmax)k\n%Iinputs+%Ooutputs (%Fmajor+%Rminor)pagefaults %Wswaps",
		//struct rusage {
		//	struct timeval ru_utime; /* user CPU time used */
		//	struct timeval ru_stime; /* system CPU time used */
		//	long   ru_maxrss;        /* maximum resident set size */
		//	long   ru_ixrss;         /* integral shared memory size */
		//	long   ru_idrss;         /* integral unshared data size */
		//	long   ru_isrss;         /* integral unshared stack size */
		//	long   ru_minflt;        /* page reclaims (soft page faults) */
		//	long   ru_majflt;        /* page faults (hard page faults) */
		//	long   ru_nswap;         /* swaps */
		//	long   ru_inblock;       /* block input operations */
		//	long   ru_oublock;       /* block output operations */
		//	long   ru_msgsnd;        /* IPC messages sent */
		//	long   ru_msgrcv;        /* IPC messages received */
		//	long   ru_nsignals;      /* signals received */
		//	long   ru_nvcsw;         /* voluntary context switches */
		//	long   ru_nivcsw;        /* involuntary context switches */

int
main(int argc, char *argv[])
{
	int pflag;
	pid_t pid;
	int status;
	struct rusage ru;
	struct timeval t0, t1;

	if (strcmp("-p", argv[1]) == 0) {
		++argv;
		pflag = 1;
	}
	if (-1 == gettimeofday(&t0, NULL))
		err(1, "gettimeofday");
	if ((pid = fork()) == -1) {
		perror("fork");
		exit(1);
	} else if (pid == 0) {
		++argv;
		if (execvp(*argv, argv) == -1) {
			perror("execvp");
			exit(1);
		}
	} else {
		if (-1 == wait4(pid, &status, 0, &ru)) {
			perror("wait4");
			exit(1);
		}
		if (-1 == gettimeofday(&t1, NULL))
			err(1, "gettimeofday");
		printf("real %f\nuser %f\nsys %f\n",
				t1.tv_sec - t0.tv_sec + (t1.tv_usec - t0.tv_usec) / 1000000.0,
				ru.ru_stime.tv_sec + ru.ru_stime.tv_usec / 1000000.0,
				ru.ru_stime.tv_sec + ru.ru_stime.tv_usec / 1000000.0);
	}

	exit(0);
}
