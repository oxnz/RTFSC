#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <signal.h>
#include <errno.h>

#include "ring_buffer.h"
#include "process_request.h"
#include "server.h"

#define NREQMAX 1024

//void dispatch_signal(int signo, siginfo_t *info, void *context);

struct server *server;

int main(int argc, char *argv[]) {
	struct server server;
	sigset_t sigset;
	siginfo_t siginfo;
	struct timespec tmo;

	{ // setup server
		server.server_socket.addr = INADDR_ANY;
		server.server_socket.port = 8000;
		server.nrequest_max = 1024;
		server.nworker_min = 16;
		server.nworker_max = 32;
		server.nevent = 10;
		server.event_tmo = 10;
		if (server_init(&server)) {
			perror("server_init");
			return -1;
		}
	}
	{ // blocking all signals
		sigfillset(&sigset);
		if (pthread_sigmask(SIG_BLOCK, &sigset, NULL) != 0) {
			perror("pthread_sigmask");
			return -1;
		}
	}
	server_startup(&server);
	{ // signal handling
		tmo.tv_sec = 0;
		tmo.tv_nsec = 100000000; // 100 ms
		sigemptyset(&sigset);
		sigaddset(&sigset, SIGINT);
		sigfillset(&sigset);
		do {
			int signo = sigtimedwait(&sigset, &siginfo, &tmo);
			//int signo = sigwaitinfo(&sigset, &siginfo);
			if (signo == -1) {
				if (errno == EAGAIN || errno == EINTR) {
					continue;
				}
				perror("sigwaitinfo");
				break;
			}
			//dispatch_signal(siginfo);
			printf("got signal %d\n", siginfo.si_signo);
			if (siginfo.si_signo == SIGINT) {
				printf("interrupted, quittingx\n");
				break;
			}
		} while (1);
	}

	printf("shuting down\n");
	server_shutdown(&server);

	return 0;
}
