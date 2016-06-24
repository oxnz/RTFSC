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

#include <sys/stat.h>
#include <errno.h>

#include <pthread.h>

#define REQMAXLEN 512
#define RSPMAXLEN 512
#define TCP_SERVICE_PORT 8000
#define THREAD_CNT 8

typedef struct sockaddr * SA;

struct taskinfo {
	pid_t pid;
	pthread_t tid;
	int listenfd;
};

int service_sendfile(int sockfd, const char *path) {
	char req[REQMAXLEN], rsp[RSPMAXLEN];
	if (-1 == fcntl(sockfd, F_SETFL, fcntl(sockfd, F_GETFL) | O_NONBLOCK))
		err(1, "fcntl");
	int fd = open(path, O_RDONLY);
	if (fd < 0) {
		if (errno == ENFILE) {
			usleep(10000);
			printf("short of nofile, sleep 10ms\n");
			return service_sendfile(sockfd, path);
		}
		warn("open");
		close(sockfd);
		return -1;
	}
	struct stat st;
	if (fstat(fd, &st) < 0) {
		warn("fstat");
		close(fd);
		close(sockfd);
		return -1;
	}
	size_t sz = snprintf(rsp, RSPMAXLEN,
			"HTTP/1.1 200 OK\r\n"
			"Content-Type: text/html; charset=utf-8\r\n"
			"Content-Length: %llu\r\n"
			"Cache-Control: private, max-age=0, proxy-revalidate, no-store, no-cache, must-revalidate\r\n"
			"Server: mhttpd/0.1\r\n"
			"\r\n",
			st.st_size
			);
	if (write(sockfd, rsp, sz) != sz)
		warn("write");
	sz = sendfile(sockfd, fd, NULL, st.st_size);
	if (sz < 0)
		warn("sendfile");
	close(fd);
	close(sockfd);
}

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
		printf("req:[%s]\n", req);
		//printf("[%s] [%s:%u] requested\n", name, addr, ntohs(cli_addr.sin_port));
		//printf("[%s] [%s:%u] requested\n%s", name, addr, ntohs(cli_addr.sin_port), req);
		n = snprintf(rsp, REQMAXLEN, "[%s] served\n", name);
		const char *path = "./index.html";
		service_sendfile(sockfd, path);
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

