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
#include <sys/sendfile.h>
#include <sys/epoll.h>

#include <errno.h>

#include <pthread.h>

#include <semaphore.h>

#define REQMAXLEN 512
#define RSPMAXLEN 512
#define URIMAXLEN 1024
#define RBMAXSIZE 1024
#define TCP_SERVICE_PORT 8000
#define NWORKER 8
#define NEVENT 10
#define NEVTMO 10

int service_sendfile(int sockfd, const char *path) {
	//path = "/home/xinyi/Downloads/cn_office_online_server_may_2016_x64_dvd_8480704.ISO";
	char req[REQMAXLEN], rsp[RSPMAXLEN];
	//if (-1 == fcntl(sockfd, F_SETFL, fcntl(sockfd, F_GETFL) | O_NONBLOCK))
	//	err(1, "fcntl");
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

int listen_addr(in_addr_t addr, in_port_t port) {
	struct sockaddr_in sockaddr;
	int sockfd;
	int listenfd, n;
	socklen_t len;
	char addr_buf[INET_ADDRSTRLEN];
	int optval = 1;

	if ((sockfd = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP)) < 0) {
		perror("socket");
		return -1;
	}
	{ // setup addr
		memset(&sockaddr, 0, sizeof(sockaddr));
		sockaddr.sin_family = AF_INET;
		sockaddr.sin_addr.s_addr = htonl(addr);
		sockaddr.sin_port = htons(port);
	}

	if (setsockopt(sockfd, SOL_SOCKET, SO_REUSEPORT, &optval, sizeof(optval)) < 0) {
		perror("setsockopt");
		return -1;
	}
	if (bind(sockfd, (struct sockaddr *) &sockaddr, sizeof(sockaddr)) < 0) {
		perror("bind");
		return -1;
	}
	if (listen(sockfd, SOMAXCONN) < 0) {
		close(sockfd);
		perror("listen");
		return -1;
	}
	if (inet_ntop(AF_INET, &sockaddr.sin_addr, addr_buf, INET_ADDRSTRLEN)) {
		printf("[%d] listening on addr: [%s:%d]\n", getpid(), addr_buf, port);
	} else
		printf("[%d] listening on addr: [misc:%d]\n", getpid(), port);

	return sockfd;
}

enum method { HEAD, GET, POST };

enum state { CONN_ESTB, HEADER_RCV };

struct request {
	int sockfd;
	char raw[REQMAXLEN];
	size_t offset;
	int state;
	enum method method;
	char *uri;
};

ssize_t read_request(struct request *req) {
	printf("read request\n");
	if (req->offset == REQMAXLEN) {
		warn("request too large");
		close(req->sockfd);
		free(req);
		return -1;
	}
	errno = 0;
	ssize_t n = read(req->sockfd, req->raw + req->offset, REQMAXLEN - req->offset);
	if (n < 0) {
		warn("read return < 0");
		close(req->sockfd);
		free(req);
	}
	if (req->state == CONN_ESTB) {
		char *header_stop = strstr(req->raw, "\n\r\n");
		char *p;
		if (header_stop) {
			req->state = HEADER_RCV;
			{ // parse header
				if (strncmp(req->raw, "GET ", 4) == 0) {
					req->method = GET;
					p = strchr(req->raw + 4, ' ');
					req->uri = strndup(req->raw + 4, p - req->raw - 4);
				} else if (strncmp(req->raw, "POST ", 5) == 0) {
					req->method = POST;
					p = strchr(req->raw + 5, ' ');
					req->uri = strndup(req->raw + 5, p - req->raw - 4);
				}
			}
			printf("[%s]\n", req->uri);
		}
	}
	printf("read %d byte(s)\n", n);
	return n;
}

struct ring_buffer {
	void* buf[RBMAXSIZE];
	size_t size;
	int in;
	int out;
	pthread_mutex_t mutex;
	sem_t nelem_sem;
	sem_t nempt_sem;
};

struct ring_buffer clients = {
	.size = RBMAXSIZE,
	.in = 0,
	.out = 0,
	.mutex = PTHREAD_MUTEX_INITIALIZER,
};

void enqueue(struct ring_buffer *buffer, void *val) {
	sem_wait(&buffer->nempt_sem);

	pthread_mutex_lock(&buffer->mutex);
	buffer->buf[buffer->in] = val;
	++(buffer->in);
	if (buffer->in == buffer->size)
		buffer->in = 0;
	pthread_mutex_unlock(&buffer->mutex);

	sem_post(&buffer->nelem_sem);
}

void* dequeue(struct ring_buffer *buffer) {
	sem_wait(&buffer->nelem_sem);

	pthread_mutex_lock(&buffer->mutex);
	void *val = buffer->buf[buffer->out];
	++(buffer->out);
	if (buffer->out == buffer->size)
		buffer->out = 0;
	pthread_mutex_unlock(&buffer->mutex);

	sem_post(&buffer->nempt_sem);
	return val;
}

void* try_dequeue(struct ring_buffer *buffer) {
	errno = 0;
	sem_trywait(&buffer->nelem_sem);
	if (errno == EAGAIN)
		return NULL;
	pthread_mutex_lock(&buffer->mutex);
	void *val = buffer->buf[buffer->out];
	++(buffer->out);
	if (buffer->out == buffer->size)
		buffer->out = 0;
	pthread_mutex_unlock(&buffer->mutex);

	sem_post(&buffer->nempt_sem);

	return val;
}

void* accpet_client(void *psockfd) {
	struct sockaddr addr;
	socklen_t addrlen;
	int sockfd;

	for (;;) {
		sockfd = accept4(*(int *)psockfd, &addr, &addrlen, SOCK_NONBLOCK);
		if (sockfd == -1) {
			perror("accept4");
			continue;
		}
		{ // count
			static int i = 0;
			++i;
			if (i % 100 == 0)
				printf("count = %d\n", i);
		}
		// enqueue client
		struct request *req = malloc(sizeof(struct request));
		req->state = CONN_ESTB;
		req->raw[0] = '\0';
		req->sockfd = sockfd;
		req->uri = NULL;
		enqueue(&clients, req);
	}
	return NULL;
}

int service(int sockfd) {
	int n;
	char addr[INET_ADDRSTRLEN];
	char req[REQMAXLEN], rsp[RSPMAXLEN];
	char name[100];

	if ((n = read(sockfd, req, REQMAXLEN)) < 0) {
		perror("read");
		if (errno == EAGAIN) {
			printf("try again later\n");
			usleep(10000);
			service(sockfd);
		}
	}
	//printf("[%s] [%s:%u] requested\n", name, addr, ntohs(cli_addr.sin_port));
	//printf("[%s] [%s:%u] requested\n%s", name, addr, ntohs(cli_addr.sin_port), req);
	const char *path = "./index.html";
	service_sendfile(sockfd, path);
	return 0;
}

void* service_client(void *arg) {
	struct epoll_event ev, events[NEVENT];
	int nfd = 0;
	int nev;
	int i;
	int sockfd;
	int epfd = epoll_create1(0);
	if (epfd == -1) {
		perror("epoll_create1");
		return NULL;
	}
	for (;;) {
		{ // pull sockfd
			struct request *req = try_dequeue(&clients);
			if (req) {
				sockfd = req->sockfd;
				ev.events = EPOLLIN | EPOLLRDHUP;
				ev.data.fd = sockfd;
				ev.data.ptr = req;
				if (epoll_ctl(epfd, EPOLL_CTL_ADD, sockfd, &ev) == 0)
					++nfd;
				else	{
					perror("epoll_ctl");
					return NULL;
				}
			}
		}
		if (nfd) { // epoll_wait
			nev = epoll_wait(epfd, events, NEVENT, NEVTMO);
			if (nev == -1) {
				perror("epoll_wait");
				return NULL;
			}
			for (i = 0; i < nev; ++i) {
				ev = events[i];
				struct request *req = ev.data.ptr;
				if (EPOLLIN & ev.events) { // read requests
					printf("events:[%x][%x]\n", ev.events, EPOLLIN | EPOLLOUT);
					ssize_t n = read_request(req);
					if (n < 0) {
						if (epoll_ctl(epfd, EPOLL_CTL_DEL, ev.data.fd, NULL) != 0) {
							perror("epoll_ctl");
							return NULL;
						}
					} else if (n == 0) {
						ev.events = EPOLLOUT;
						if (epoll_ctl(epfd, EPOLL_CTL_MOD, ev.data.fd, NULL) != 0) {
							perror("epoll_ctl");
							return NULL;
						}
					}
				}
				if (EPOLLOUT & ev.events) { // write response
					printf("need send resp\n%s]", req->raw);
					// send_response();
				}
				if (EPOLLRDHUP & ev.events) { // request read EOF, start response
					ev.events = EPOLLOUT;
					if (epoll_ctl(epfd, EPOLL_CTL_MOD, ev.data.fd, &ev) != 0) {
						perror("epoll_ctl");
						return NULL;
					}
				}
				if (EPOLLHUP & ev.events) { // HUP
					printf("HUP\n");
					close(ev.data.fd);
					if (epoll_ctl(epfd, EPOLL_CTL_DEL, ev.data.fd, NULL) == -1) {
						perror("epoll_ctl");
						return NULL;
					}
				}
			}
		} // epoll_wait end
	}
}

int main(int argc, char *argv[]) {
	int sockfd;
	{ // setup ring buffer
		sem_init(&clients.nelem_sem, 0, 0);
		sem_init(&clients.nempt_sem, 0, RBMAXSIZE);
	}
	{ // setup consumer
		pthread_t workers[NWORKER];
		int i;
		for (i = 0; i < NWORKER; ++i) {
			if (pthread_create(&workers[i], NULL, service_client, NULL) != 0) {
				perror("pthread_create");
			}
		}
	}
	{ // setup sockets
		in_addr_t addr = INADDR_ANY;
		in_port_t port = TCP_SERVICE_PORT;
		sockfd = listen_addr(addr, port);
	}
	{ // setup producer
		pthread_t accept_thread;
		if (pthread_create(&accept_thread, NULL, accpet_client, &sockfd) != 0) {
			perror("pthread_create");
			return -1;
		}
		pthread_join(accept_thread, NULL);
	}
	close(sockfd);
	return 0;
}
