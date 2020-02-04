#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cerrno>

#include <unistd.h>
#include <netinet/tcp.h>
#include <sys/types.h>
#include <sys/stat.h>

#include "process_request.h"
#include "server.h"
#include "processor.h"

processor::processor(ident id, const server& server) : processor(id, server, server.config.req_queue_size()) {}

processor::processor(ident id, const server& server, size_t capacity)
    :
    m_id(id),
    m_name(std::string("processor-") + std::to_string(id)),
    m_server(server),
    m_evloop(capacity, to_timespec(m_server.config.event_timeout())),
    m_requests(capacity),
    m_capacity(capacity) {
    syslog(LOG_DEBUG, "[%s] ctor", m_name.c_str());
#ifdef __linux__
    require(m_epfd != -1, "epoll_create");
    struct epoll_event ev;
    ev.events = EPOLLIN;
    ev.data.u64 = 0;
    if (-1 == epoll_ctl(m_epfd, EPOLL_CTL_ADD, server.socket.sockfd, &ev)) {
        syslog(LOG_ERR, "[%s] epoll_ctl: %s", m_name.c_str(), strerror(errno));
        throw std::runtime_error("epoll_ctl");
    }
#elif __APPLE__
    m_requests[0].sockfd = m_server.socket.sockfd;
    m_requests[0].addr = m_server.socket.addr;
    m_evloop(m_server.socket.sockfd, EF_READABLE, 0);
#endif
    for (size_t i = 1; i < m_capacity; ++i)
        m_unused.insert(i);
}

processor::~processor() {
    syslog(LOG_DEBUG, "[%s] dtor", m_name.c_str());
}

void processor::operator()() {
    syslog(LOG_INFO, "[%s] started", m_name.c_str());
    while (m_server.running()) {
        syslog(LOG_DEBUG, "[%s] stat: free: %lu, capacity: %lu", m_name.c_str(), m_unused.size(), m_capacity);
        try {
            sleep(2);
            m_evloop(*this);
        } catch (std::exception& ex) {
            syslog(LOG_ERR, "[%s] aborted: %s (%m)", m_name.c_str(), ex.what());
        }
    }
    syslog(LOG_INFO, "[%s] stopped", m_name.c_str());
}

bool processor::operator()(const event& ev) {
    syslog(LOG_DEBUG, "[%s] processing %s", m_name.c_str(), ev.repr().c_str());
    size_t idx = ev.ptr();
    if (idx == 0) {
        if (ev.readable() && !full() && !m_evloop.full()) {
            struct sockaddr_in addr;
            socklen_t addrlen = sizeof(addr);
            do {
                int sockfd = accept(m_server.socket.sockfd, (struct sockaddr*)&addr, &addrlen);
                if (-1 == sockfd) {
                    if (errno != EAGAIN)
                        syslog(LOG_ERR, "[%s] accept: %m", m_name.c_str());
                    break;
                }
                nonblock(sockfd);
                request req(sockfd);
                req.addr = addr;
                syslog(LOG_DEBUG, "[%s] new connection: %s", m_name.c_str(), repr(addr).c_str());
                { // enq req
                    idx = *m_unused.begin();
                    m_evloop(sockfd, EF_READABLE|EF_WRITABLE, idx);
                    m_unused.erase(idx);
                    std::swap(req, m_requests[idx]);
                }
            } while (!full());
        }
        return true;
    }
    bool rm = false;
    request& req = m_requests[idx];
    if (ev.error()) {
        rm = true;
        syslog(LOG_DEBUG, "[%s] error event (fd: %d)", m_name.c_str(), ev.fd());
    }
    if (ev.readable()) {
        syslog(LOG_DEBUG, "[%s] readable", m_name.c_str());
        req.readable = true;
        if (process_request(req) != 0) {
            syslog(LOG_ERR, "process_request");
            rm = true;
        }
    }
    if (ev.writable()) {
        syslog(LOG_DEBUG, "[%s] writable", m_name.c_str());
        req.writable = true;
        if (process_request(req) != 0) {
            syslog(LOG_ERR, "process_request");
            rm = true;
        }
    }
    if (ev.eof()) {
        rm = true;
        syslog(LOG_DEBUG, "[%s] eof", m_name.c_str());
    }
    if (req.state == CONN_ABORTED) {
        syslog(LOG_DEBUG, "[%s] conn_aborted", m_name.c_str());
        rm = true;
    }
    if (req.state == RESP_SENT) {
        syslog(LOG_DEBUG, "[%s] resp_sent", m_name.c_str());
        rm = true;
    }
    if (rm) {
        syslog(LOG_DEBUG, "[%s] finish processing %s", m_name.c_str(), req.repr().c_str());
        close(req.sockfd);
        m_unused.insert(idx);
        // delete m_requests[ev.data.u64];
    }
    return true;
}

#ifdef __linux__

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
#endif


