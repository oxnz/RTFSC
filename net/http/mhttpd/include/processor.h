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
#include "event.h"

#include <fcntl.h>

#include <pthread.h>

#include <semaphore.h>
#include <unordered_set>

#include "process_request.h"
#include "ring_buffer.h"
#include "server.h"

struct processor {
public:
    processor(const std::string& name, server& server): processor(name, server, 128) {}
    processor(const std::string& name, server& server, size_t capacity): m_name(name), m_server(server), m_epfd(epoll_create(1)), m_capacity(capacity),
        m_events(new struct epoll_event[capacity]), m_requests(capacity) {
        require(m_epfd != -1, "epoll_create");
        for (size_t i = 1; i < m_capacity; ++i)
            m_unused.insert(i);
        struct epoll_event ev;
        ev.events = EPOLLIN;
        ev.data.u64 = 0;
        if (-1 == epoll_ctl(m_epfd, EPOLL_CTL_ADD, server.socket.sockfd, &ev)) {
            syslog(LOG_ERR, "[%s] epoll_ctl: %s", m_name.c_str(), strerror(errno));
            throw std::runtime_error("epoll_ctl");
        }
    }
    ~processor() {
        //delete[] m_events;
    }
    void operator()() {
        try {
            syslog(LOG_INFO, "[%s] started", m_name.c_str());
            while (m_server.state != SVR_STOPPING_WORKER) {
                syslog(LOG_DEBUG, "[%s] stat: free: %lu, capacity: %lu", m_name.c_str(), m_unused.size(), m_capacity);
                process();
            }
            syslog(LOG_INFO, "[%s] stopped", m_name.c_str());
        } catch (std::exception& ex) {
            syslog(LOG_ERR, "[%s] aborted: %s", m_name.c_str(), ex.what());
        }
    }
private:
    void enq(request& req) {
        syslog(LOG_DEBUG, "[%s] new request", m_name.c_str());
        size_t idx = *m_unused.begin();
        m_ev.data.u64 = idx;
        m_ev.events = EPOLLIN | EPOLLOUT | EPOLLRDHUP | EPOLLET;
        if (epoll_ctl(m_epfd, EPOLL_CTL_ADD, req.sockfd, &m_ev) == -1) {
            syslog(LOG_ERR, "[%s] epoll_ctl(%d, %d): %s", m_name.c_str(), m_epfd, req.sockfd, strerror(errno));
            throw std::runtime_error("epoll_ctl");
        }
        m_unused.erase(idx);
        std::swap(req, m_requests[idx]);
    }
    void process(struct epoll_event& ev) {
        size_t idx = ev.data.u64;
        if (idx == 0) {
				if (!(EPOLLIN & ev.events)) {
                syslog(LOG_ERR, "[%s][listener][event]: %x", m_name.c_str(), ev.events);
				return;
				}
            while (!full()) {
                struct sockaddr_in addr;
                socklen_t addrlen = sizeof(addr);
                int sockfd = accept4(m_server.socket.sockfd, (struct sockaddr*)&addr, &addrlen, SOCK_NONBLOCK | SOCK_CLOEXEC);
				if (-1 == sockfd) {
						syslog(LOG_ERR, "[%s]accept: %s", m_name.c_str(), strerror(errno));
						break;
				}
                request req(sockfd);
                req.addr = std::to_string(htons(addr.sin_port));
                enq(req);
            }
            return;
        }
        bool rm = false;
        request& req = m_requests[idx];
        syslog(LOG_DEBUG, "[%s] event = (fd: %d, events: %x)", m_name.c_str(), req.sockfd, ev.events);
        if (EPOLLIN & ev.events) {
            ev.events &= ~EPOLLIN;
            syslog(LOG_DEBUG, "[%s] XXX0", m_name.c_str());
            req.readable = true;
            if (process_request(req) != 0) {
                syslog(LOG_ERR, "process_request");
                rm = true;
            }
        }
        if (EPOLLOUT & ev.events) {
            ev.events &= ~EPOLLOUT;
            syslog(LOG_DEBUG, "[%s] XXX1", m_name.c_str());
            req.writable = true;
            if (process_request(req) != 0) {
                syslog(LOG_ERR, "process_request");
                rm = true;
            }
        }
        if ((EPOLLRDHUP | EPOLLHUP ) & ev.events) {
            ev.events &= ~EPOLLRDHUP;
            ev.events &= ~EPOLLHUP;
            syslog(LOG_DEBUG, "[%s] XXX2", m_name.c_str());
            rm = true;
        }
        if (ev.events) {
            syslog(LOG_ERR, "[%s] unrecognized events: [%x]", m_name.c_str(), ev.events);
            rm = true;
        }
        if (req.state == CONN_ABORTED) {
            rm = true;
        }
        if (req.state == RESP_SENT) {
            rm = true;
        }
        if (rm) {
            syslog(LOG_DEBUG, "[%s] close req: %s", m_name.c_str(), req.addr.c_str());
            close(req.sockfd);
            m_unused.insert(idx);
            // delete m_requests[ev.data.u64];
        }
    }
    void process() {
        int nfd = epoll_wait(m_epfd, m_events, m_capacity, m_server.config.event_timeout());
        if (nfd == -1) {
            syslog(LOG_ERR, "[%s] epoll_wait: %s", m_name.c_str(), strerror(errno));
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
    const std::string m_name;
    server& m_server;
    int m_epfd;
    std::unordered_set<size_t> m_unused;
    struct epoll_event m_ev;
    struct epoll_event* m_events;
    std::vector<request> m_requests;
    size_t m_capacity;
};

#endif//PROCESSOR_H
