#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include <errno.h>

#include <unistd.h>

#include <err.h>

#include <netinet/tcp.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <sys/epoll.h>
#include <sys/sendfile.h>

#include <fcntl.h>

#include <pthread.h>

#include <semaphore.h>

#include "process_request.h"
#include "ring_buffer.h"
#include "server.h"

#define REQMAXLEN 512
#define RSPMAXLEN 512
#define URIMAXLEN 1024

struct request* request_create(int sockfd) {
	struct request* req = malloc(sizeof(struct request));
	if (req) {
		req->sockfd = sockfd;
		req->raw[0] = '\0';
		req->offset = 0;
		req->state = CONN_ESTABLISHED;
		req->uri = NULL;
	}
	return req;
}

void request_destroy(struct request* req) {
	free(req);
}

void* accept_request(struct server *server) {
	int sockfd;
	int epfd;
	struct epoll_event ev, evs[10];
	int nev;
	int tmo = 10;
	int optval = 1;
	int i;

	if ((epfd = epoll_create(1)) == -1) {
		perror("epoll_create");
		return NULL;
	}
	ev.data.fd = server->server_socket.sockfd;
	ev.events = EPOLLIN;
	if (epoll_ctl(epfd, EPOLL_CTL_ADD, server->server_socket.sockfd, &ev) == -1) {
		close(epfd);
		perror("epoll_ctl");
		return NULL;
	}
	for (;server->state != SVR_STOPPING_LISTENER;) {
		nev = epoll_wait(epfd, evs, 1, tmo);
		if (nev == -1) {
			perror("epoll_wait");
			close(epfd);
			return NULL;
		}
		for (i = 0; i < nev; ++i) {
			ev = evs[i];
			if (EPOLLIN & ev.events) {
				sockfd = accept(ev.data.fd, NULL, NULL);
				if (sockfd == -1) {
					perror("accept");
					continue;
				}
				// set non-blocking
				if (-1 == fcntl(sockfd, F_SETFL, fcntl(sockfd, F_GETFL) | O_NONBLOCK)) {
					perror("fcntl");
					close(sockfd);
					continue;
				}
				if (1) { // set nodelay
					if (setsockopt(sockfd, SOL_TCP, TCP_NODELAY, &optval, sizeof(optval))) {
						perror("setsockopt");
					}
				}
				// enqueue request
				struct request *req = request_create(sockfd);
				if (req) {
					ring_buffer_enqueue(server->requests, req);
				} else {
					perror("request_create");
					return NULL;
				}
			} else {
				close(sockfd);
				printf("unrognized event: %x\n", ev.events);
				return NULL;
			}
		}
	}
	return NULL;
}

ssize_t read_request(struct request *req) {
	ssize_t n;
	ssize_t sum = -1;
	if (req->offset == REQMAXLEN) {
		warn("request too large");
		req->state = CONN_ABORTED;
		return -1;
	}
	do {
		n = read(req->sockfd, req->raw + req->offset, REQMAXLEN - req->offset);
		if (n >= 0) {
			if (sum == -1)
				sum = n;
			else
				sum += n;
			req->offset += n;
			req->readable = (n != 0);
		} else {
			req->readable = 0;
			if (errno != EAGAIN) {
				perror("read");
				req->state = CONN_ABORTED;
				return -1;
			}
		}
	} while (req->readable);
	if (req->state == CONN_ESTABLISHED) {
		char *p = strstr(req->raw, "\n\r\n");
		if (p) { // parse header
			if (strncmp(req->raw, "GET ", 4) == 0) {
				req->method = GET;
				req->state = REQ_RCVD;
				p = strchr(req->raw + 4, ' ');
				req->uri = strndup(req->raw + 4, p - req->raw - 4);
			} else if (strncmp(req->raw, "POST ", 5) == 0) {
				req->method = POST;
				req->state = READING_REQ_BODY;
				p = strchr(req->raw + 5, ' ');
				req->uri = strndup(req->raw + 5, p - req->raw - 4);
			} else if (strncmp(req->raw, "HEAD ", 5) == 0) {
				req->method = HEAD;
				req->method = REQ_RCVD;
				p = strchr(req->raw + 5, ' ');
				req->uri = strndup(req->raw + 5, p - req->raw - 4);
			} else {
				req->state = CONN_ABORTED;
				printf("unsupported request method\n");
				return -1;
			}
		}
	}
	return sum >= 0 ? sum : -1;
}

ssize_t send_header(struct request *req) {
	struct response *resp = &req->resp;
	ssize_t n, sum = -1;

	do {
		n = write(req->sockfd, resp->header + resp->header_offset,
				resp->header_length);
		//printf("[%llx] send header %d/%d bytes\n", pthread_self(), n, resp->header_length);
		if (n >= 0) {
			if (sum == -1)
				sum = n;
			else
				sum += n;
			req->writable = (n != 0);
			resp->header_offset += n;
			resp->header_length -= n;
			if (resp->header_length == 0) {
				if (resp->body_length == 0)
					req->state = RESP_SENT;
				else
					req->state = SENDING_RESP_BODY;
				break;
			}
		} else {
			req->writable = 0;
			if (errno != EAGAIN) {
				perror("write");
				return -1;
			}
		}
	} while (req->writable);
	return sum >= 0 ? sum : -1;
}

ssize_t send_body(struct request *req) {
	struct response *resp = &req->resp;
	ssize_t n, sum = -1;

	do {
		n = sendfile(req->sockfd, resp->fd, &resp->body_offset, resp->body_length);
		if (n >= 0) {
			if (sum == -1)
				sum = n;
			else
				sum += n;
			req->writable = (n != 0);
			resp->body_length -= n;
			if (resp->body_length == 0) {
				close(resp->fd);
				req->state = RESP_SENT;
				break;
			}
		} else {
			req->writable = 0;
			if (errno != EAGAIN) {
				perror("sendfile");
				close(resp->fd);
				req->state = CONN_ABORTED;
			}
		}
	} while (req->writable);
	//printf("[%llx] send body %d sent/%d offset/%d remains\n", pthread_self(), n, resp->body_offset, resp->body_length);
	return sum >= 0 ? sum : -1;
}

int build_resp(struct request* req) {
	struct response *resp = &req->resp;
	resp->header_offset = 0;
	char *fpath;
	if (strcmp("/", req->uri) == 0)
		fpath = "index.html";
	else if (strlen(req->uri) > 0 && req->uri[0] == '/') {
		fpath = req->uri + 1;
	} else
		fpath = "404.html";
	resp->fd = open(fpath, O_RDONLY | O_NONBLOCK);
	resp->content_type = "text/html";
	resp->charset = "utf-8";
	if (req->resp.fd != -1) {
		resp->code = 200;
		resp->reason = "OK";
		struct stat st;
		if (fstat(resp->fd, &st) < 0) {
			warn("fstat");
			resp->code = 500;
			resp->reason = "Internal Error";
			close(resp->fd);
		}
		resp->body_offset = 0;
		resp->body_length = st.st_size;
	} else {
		if (errno == EACCES) {
			resp->code = 403;
			resp->reason = "Forbidden";
		} else if (errno == EEXIST) {
			resp->code = 404;
			resp->reason = "Not Found";
		} else {
			resp->code = 500;
			resp->reason = "Internal Server Error";
		}
	}
	if (resp->code != 200)
		resp->body_length = 0;
	int n = sprintf(resp->header, "HTTP/1.1 %d %s\r\n",
			resp->code, resp->reason);
	n += sprintf(resp->header + n, "Server: mhttpd/0.1\r\n");
	n += sprintf(resp->header + n, "Content-Type: %s; charset=%s\r\n",
			resp->content_type, resp->charset);
	n += sprintf(resp->header + n,
			"Cache-Control: private, max-age=0, proxy-revalidate, no-store, no-cache, must-revalidate\r\n");
	n += sprintf(resp->header + n, "Content-Length: %llu\r\n",
			resp->body_length);
	n += sprintf(resp->header + n, "\r\n");
	resp->header_length = n;
	req->state = SENDING_RESP_HEADER;
	// go through to send_header
	return 0;
}

/*
 * request processing
 * return:
 * 	0 success
 * 	other failure
 */
int process_request(struct request *req) {
	struct response *resp = &req->resp;
	int stop = 1;
	enum state old_state;
	do {
		old_state = req->state;
		switch (req->state) {
			case CONN_ESTABLISHED:
			case READING_REQ_HEADER:
			case READING_REQ_BODY:
				if (req->readable == 0)
					return 0;
				if (read_request(req) < 0)
					return -1;
				if (req->state != old_state) {
					stop = 0;
					break;
				}
				if (req->readable == 0)
					return 0;
			case REQ_RCVD:
				build_resp(req);
			case SENDING_RESP_HEADER:
				if (req->writable == 0)
					return 0;
				if (send_header(req) < 0)
					return -1;
				if (req->state != old_state) {
					stop = 0;
					break;
				}
				if (req->writable == 0)
					return 0;
			case SENDING_RESP_BODY:
				if (req->writable == 0)
					return 0;
				if (send_body(req) < 0)
					return -1;
				if (req->state != old_state) {
					stop = 0;
					break;
				}
				if (req->writable == 0)
					return 0;
			case RESP_SENT:
				close(req->sockfd);
				return 0;
			case CONN_ABORTED:
				return -1;
			default:
				printf("misc status: %d\n", req->state);
				return -1;
		}
	} while (!stop);
	return 0;
}

void* dispatch_request(struct server *server) {
	struct epoll_event ev, events[NEVENT];
	int nfd = 0;
	int nev;
	int i;
	int sockfd;
	int rm;
	int epfd;
	struct request *req;

	epfd = epoll_create(1);
	if (epfd == -1) {
		perror("epoll_create");
		return NULL;
	}
	for (;server->state != SVR_STOPPING_WORKER || nfd > 0;) {
		do { // pull sockfd
			req = ring_buffer_try_dequeue(server->requests);
			if (req) {
				ev.events = EPOLLIN | EPOLLOUT | EPOLLRDHUP | EPOLLET;
				ev.data.ptr = req;
				if (epoll_ctl(epfd, EPOLL_CTL_ADD, req->sockfd, &ev) == 0) {
					++nfd;
					printf("nfd = %d\n", nfd);
				} else {
					perror("epoll_ctl");
					return NULL;
				}
			} else break;
		} while (nfd < 5);
		{ // epoll_wait
			nev = epoll_wait(epfd, events, NEVENT, server->event_tmo);
			if (nev == -1 && errno != EINTR) {
				perror("epoll_wait");
				return NULL;
			}
			//printf("[%llx] nfd = %d, nevents = %d\n", pthread_self(), nfd, nev);
			for (i = 0; i < nev; ++i) {
				rm = 0;
				ev = events[i];
				req = ev.data.ptr;
				//printf("[%llx] event = (fd: %d, events: %x)\n", pthread_self(), req->sockfd, ev.events);
				if ((EPOLLRDHUP | EPOLLHUP ) & ev.events) {
					rm = 1;
				} else if ((EPOLLIN | EPOLLOUT) & ev.events) {
					req->readable = ev.events & EPOLLIN;
					req->writable = ev.events & EPOLLOUT;
					if (process_request(req) != 0) {
						warn("process_request");
						rm = 1;
					}
				} else {
					printf("unrecognized events: [%x]\n", ev.events);
					rm = 1;
				}
				if (req->state == CONN_ABORTED) {
					rm = 1;
				} else if (req->state == RESP_SENT) {
					rm = 1;
				}
				if (rm) {
					close(req->sockfd);
					--nfd;
					request_destroy(req);
				}
			}
		} // epoll_wait end
	}
	printf("dispatch quit\n");
	return NULL;
}
