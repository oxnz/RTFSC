#ifndef PROCESSOR_H
#define PROCESSOR_H
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
#include <unordered_set>

#include "process_request.h"
#include "ring_buffer.h"
#include "server.h"

struct processor {
public:
		processor(server& server): processor(server, 128) {}
		processor(server& server, size_t capacity): m_server(server), m_epfd(epoll_create(1)), m_capacity(capacity),
		m_events(new struct epoll_event[capacity]), m_requests(capacity) {
				require(m_epfd != -1, "epoll_create");
				for (size_t i = 1; i < m_capacity; ++i)
						m_unused.insert(i);
				struct epoll_event ev;
				ev.events = EPOLLIN;
                ev.data.u64 = 0;
				if (-1 == epoll_ctl(m_epfd, EPOLL_CTL_ADD, server.socket.sockfd, &ev)) {
						syslog(LOG_ERR, "[worker] epoll_ctl: %s", strerror(errno));
						throw std::runtime_error("epoll_ctl");
				}
		}
		~processor() {
				//delete[] m_events;
		}
		void operator()() {
				try {
						syslog(LOG_INFO, "[worker-%lx] started", pthread_self());
						while (m_server.state != SVR_STOPPING_WORKER) {
								syslog(LOG_DEBUG, "[worker-%lx] stat: free: %lu, capacity: %lu", pthread_self(), m_unused.size(), m_capacity);
								process();
						}
						syslog(LOG_INFO, "[worker-%lx] stopped", pthread_self());
				} catch (std::exception& ex) {
						syslog(LOG_ERR, "[worker-%lx] aborted: %s", pthread_self(), ex.what());
				}
		}
private:
		void enq(request& req) {
				syslog(LOG_DEBUG, "[worker] new request");
				size_t idx = *m_unused.begin();
				m_ev.data.u64 = idx;
				m_ev.events = EPOLLIN | EPOLLOUT | EPOLLRDHUP | EPOLLET;
				if (epoll_ctl(m_epfd, EPOLL_CTL_ADD, req.sockfd, &m_ev) == -1) {
						syslog(LOG_ERR, "[worker] epoll_ctl(%d, %d): %s", m_epfd, req.sockfd, strerror(errno));
						throw std::runtime_error("epoll_ctl");
				}
				m_unused.erase(idx);
				std::swap(req, m_requests[idx]);
		}
		void process(struct epoll_event& ev) {
				size_t idx = ev.data.u64;
				if (idx == 0) {
						if (!full() && EPOLLIN & ev.events) {
								struct sockaddr_in addr;
								socklen_t addrlen = sizeof(addr);
								int sockfd = accept4(m_server.socket.sockfd, (struct sockaddr*)&addr, &addrlen, SOCK_NONBLOCK | SOCK_CLOEXEC);
								request req(sockfd);
								req.addr = std::to_string(htons(addr.sin_port));
								enq(req);
						} else {
								syslog(LOG_ERR, "[listener][event]: %x", ev.events);
						}
						return;
				}
				bool rm = false;
				request& req = m_requests[idx];
				syslog(LOG_DEBUG, "thread-[%lx] event = (fd: %d, events: %x)", pthread_self(), req.sockfd, ev.events);
				if (EPOLLIN & ev.events) {
						ev.events &= ~EPOLLIN;
						syslog(LOG_DEBUG, "XXX0");
						req.readable = true;
						if (process_request(req) != 0) {
								syslog(LOG_ERR, "process_request");
								rm = true;
						}
				}
				if (EPOLLOUT & ev.events) {
						ev.events &= ~EPOLLOUT;
						syslog(LOG_DEBUG, "XXX1");
						req.writable = true;
						if (process_request(req) != 0) {
								syslog(LOG_ERR, "process_request");
								rm = true;
						}
				}
				if ((EPOLLRDHUP | EPOLLHUP ) & ev.events) {
						ev.events &= ~EPOLLRDHUP;
						ev.events &= ~EPOLLHUP;
						syslog(LOG_DEBUG, "XXX2");
						rm = true;
				}
				if (ev.events) {
						syslog(LOG_DEBUG, "XXX3");
						syslog(LOG_ERR, "unrecognized events: [%x]", ev.events);
						rm = true;
				}
				if (req.state == CONN_ABORTED) {
						rm = true;
				} else if (req.state == RESP_SENT) {
						rm = true;
				}
				if (rm) {
						syslog(LOG_DEBUG, "close req: %s", req.addr.c_str());
						close(req.sockfd);
						m_unused.insert(idx);
						// delete m_requests[ev.data.u64];
				}
		}
		void process() {
				int nfd = epoll_wait(m_epfd, m_events, m_capacity, m_server.event_tmo);
				if (nfd == -1) {
						syslog(LOG_ERR, "[epoll_wait]: %s", strerror(errno));
						return;
				}
				for (int i = 0; i < nfd; ++i) process(m_events[i]);
		}
		size_t capacity() const { return m_capacity; }
		size_t size() const { return m_capacity - m_unused.size(); }
		size_t size_to_full() const { return m_unused.size(); }
		bool empty() const { return size() == 0; }
		bool full() const { return m_unused.empty(); }
private:
		server& m_server;
		int m_epfd;
		std::unordered_set<size_t> m_unused;
		struct epoll_event m_ev;
		struct epoll_event* m_events;
		std::vector<request> m_requests;
		size_t m_capacity;
};

#endif//PROCESSOR_H
