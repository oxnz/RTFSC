#ifndef CONNECTION_H
#define CONNECTION_H

#include "config.h"
#include "helper.h"

class connection {
public:
    void open() {
    }
    void close() {
        ensure(0 == close(m_sockfd), strerror(errno));
    }
    bool read(request& req) {
    }
    void write(response& resp) {
    }
private:
    struct sockaddr_in m_addr;
    char m_buffer[4096];
    int m_offset;
    int m_sockfd;
};

#endif//CONNECTION_H
