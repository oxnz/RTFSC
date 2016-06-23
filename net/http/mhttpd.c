#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <sys/socket.h>

#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>

#include <pthread.h>

#define REQMAXLEN 512
#define RSPMAXLEN 512
#define TCP_SERVICE_PORT 8000
#define THREAD_CNT 5

char buf[10240];
size_t bufl;

typedef struct sockaddr * SA;

struct taskinfo {
	pid_t pid;
	pthread_t tid;
	int listenfd;
};

void* service(void *arg) {
	struct taskinfo *ti = arg;
	int listenfd = ti->listenfd;
	pid_t pid = ti->pid;
	pthread_t tid = ti->tid;
	struct sockaddr_in cli_addr;
	int len;
	int sockfd;
	int n;
	char addr[INET_ADDRSTRLEN];
	char req[REQMAXLEN], rsp[RSPMAXLEN];
	char name[100];
	snprintf(name, 100, "%d:%lld", ti->pid, ti->tid);

	for (;;) {
		len = sizeof(cli_addr);
		sockfd = accept(listenfd, (SA) &cli_addr, &len);
		if (sockfd  < 0) {
			perror("accept");
			err(1, "accept");
		}
		//printf("[%s] accepted\n", name);
		if (inet_ntop(AF_INET, &cli_addr.sin_addr, addr,
					INET_ADDRSTRLEN) == NULL)
			err(1, "inet_ntop");
		if ((n = read(sockfd, req, REQMAXLEN)) < 0)
			err(1, "read");
		//printf("[%s] [%s:%u] requested\n", name, addr, ntohs(cli_addr.sin_port));
		//printf("[%s] [%s:%u] requested\n%s", name, addr, ntohs(cli_addr.sin_port), req);
		n = snprintf(rsp, REQMAXLEN, "[%s] served\n", name);
		if (-1 == fcntl(sockfd, F_SETFL, fcntl(sockfd, F_GETFL) | O_NONBLOCK))
			err(1, "fcntl");
		//if (write(sockfd, rsp, n) != n)
		if (write(sockfd, buf, bufl) != bufl)
			err(1, "write");
		close(sockfd);
	}
}

void process(int listenfd) {
	struct taskinfo ti[THREAD_CNT];
	int i;
	for (i = 0; i < THREAD_CNT; ++i) {
		ti[i].listenfd = listenfd;
		ti[i].pid = getpid();
		if (pthread_create(&ti[i].tid, NULL, service, &ti[i]))
			err(1, "pthread_create");
	}
	for (i = 0; i < THREAD_CNT; ++i) {
		if (pthread_join(ti[i].tid, NULL))
			err(1, "pthread_join");
	}
}

int main(int argc, char *argv[]) {
	struct sockaddr_in srv_addr;
	int listenfd, n, len;
	char addr[INET_ADDRSTRLEN];
	int optval = 1;

	if ((listenfd = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP)) < 0)
		err(1, "socket");
	memset(&srv_addr, 0, sizeof(srv_addr));
	srv_addr.sin_family = AF_INET;
	srv_addr.sin_addr.s_addr = htonl(INADDR_ANY);
	srv_addr.sin_port = htons(TCP_SERVICE_PORT);

	if (setsockopt(listenfd, SOL_SOCKET, SO_REUSEPORT, &optval, sizeof(optval)) < 0)
		err(1, "setsockopt");
	if (bind(listenfd, (SA) &srv_addr, sizeof(srv_addr)) < 0)
		err(1, "bind");
	if (listen(listenfd, SOMAXCONN) < 0)
		err(1, "listen");
	if (inet_ntop(AF_INET, &srv_addr.sin_addr, addr, INET_ADDRSTRLEN)
			== NULL)
		err(1, "inet_ntop");
	printf("[%d] listening on addr: [%s:%d]\n", getpid(), addr, TCP_SERVICE_PORT);

	{ // setup index.html
		int fd = open("./index.html", O_RDONLY);
		if (fd < 0) {
			warn("open");
			snprintf(buf, 10, "MESSAGE\n");
			bufl = strlen(buf);
		} else {
			if ((bufl = read(fd, buf, 10240)) <= 0)
				err(1, "read");
			close(fd);
		}
	}

	pid_t pid = getpid();
	int i;

	for (i = 0; i < 8; ++i) {
		if (getpid() == pid)
			if (fork() < 0)
				err(1, "fork");
	}
	if (getpid() != pid)
		process(listenfd);

	wait();

	return 0;
}

