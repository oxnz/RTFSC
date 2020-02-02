#ifndef LISTENER_H
#define LISTENER_H
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
#include "request_pool.h"

class listener {
		int epfd;
public:
		listener() : epfd(epoll_create(1)) {
				require(epfd != -1, "epoll_create");
		}
		void operator()(server* p) {
				int sockfd;
				struct epoll_event ev;
				int nev;
				int tmo = 10;
				int optval = 1;
				int i;
				server& server = *p;

				ev.events = EPOLLIN;
				ev.data.fd = server.socket.sockfd;
				if (epoll_ctl(epfd, EPOLL_CTL_ADD, server.socket.sockfd, &ev) == -1) {
						close(epfd);
						syslog(LOG_ERR, "[listener] epoll_ctl(%d, CTL_ADD, %d): %s", epfd, server.socket.sockfd, strerror(errno));
						throw std::runtime_error("[listener] epoll_ctl");
				}
				for (;server.state != SVR_STOPPING_LISTENER;) {
						nev = epoll_wait(epfd, &ev, 1, tmo);
						if (nev == -1) {
								close(epfd);
								syslog(LOG_ERR, "[listener] epoll_wait: %s", strerror(errno));
								throw std::runtime_error("epoll_wait");
						}
						if (nev == 1) {
								if (EPOLLIN & ev.events) {
										sockfd = accept(ev.data.fd, NULL, NULL);
										if (sockfd == -1) {
												syslog(LOG_ERR, "[listener]: accept: %s", strerror(errno));
												continue;
										}
										// set non-blocking
										if (-1 == fcntl(sockfd, F_SETFL, fcntl(sockfd, F_GETFL) | O_NONBLOCK)) {
												syslog(LOG_ERR, "[listener] fcntl: %s", strerror(errno));
												close(sockfd);
												continue;
										}
										if (true) { // set nodelay
												if (setsockopt(sockfd, SOL_TCP, TCP_NODELAY, &optval, sizeof(optval))) {
														syslog(LOG_ERR, "[listener] setsockopt: %s", strerror(errno));
												}
										}
										// enqueue request
										request req(sockfd);
										server.requests.enqueue(req);
										syslog(LOG_DEBUG, "[listener] new request: %s", req.addr.c_str());
								} else {
										close(sockfd);
										syslog(LOG_ERR, "unrognized event: %x", ev.events);
										return;
								}
						}
				}
		}
};

#endif//LISTENER_H
