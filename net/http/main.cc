#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <errno.h>

#include "ring_buffer.h"
#include "process_request.h"
#include "server.h"

#define NREQMAX 1024

//void dispatch_signal(int signo, siginfo_t *info, void *context);

int main(int argc, char *argv[]) {

	try {
			/*
			*/
			struct server server;
			server.serve();
	} catch (std::exception& e) {
			std::cerr << "exception: " << e.what() << std::endl;
			syslog(LOG_ERR, "[exception]: %s", e.what());
			return -1;
	}

	return 0;
}
