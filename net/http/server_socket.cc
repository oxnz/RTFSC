/*
 * Filename:	server_socket.c
 *
 * Author:		Oxnz
 * Email:		yunxinyi@gmail.com
 * Created:		2016-06-26 21:08:47 CST
 * Last-update:	2016-06-26 21:08:47 CST
 * Description: anchor
 *
 * Version:		0.0.1
 * Revision:	[NONE]
 * Revision history:	[NONE]
 * Date Author Remarks:	[NONE]
 *
 * License:
 * Copyright (c) 2016 Oxnz
 *
 * Distributed under terms of the [LICENSE] license.
 * [license]
 *
 */

#include <string.h>
#include <unistd.h>
#include "server.h"
#include "server_socket.h"

server_socket::server_socket(in_addr_t addr, in_port_t port) {
	struct sockaddr_in sockaddr;
	char addr_buf[INET_ADDRSTRLEN] = "misc";
	int optval = 1;

	if ((sockfd = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP)) < 0) {
		syslog(LOG_ERR, "[socket]: %s", strerror(errno));
		throw std::runtime_error("socket");
	}
	{ // setup addr
		memset(&sockaddr, 0, sizeof(sockaddr));
		sockaddr.sin_family = AF_INET;
		sockaddr.sin_addr.s_addr = htonl(addr);
		sockaddr.sin_port = htons(port);
	}

	if (setsockopt(sockfd, SOL_SOCKET, SO_REUSEPORT,
				&optval, sizeof(optval)) < 0) {
		syslog(LOG_ERR, "[setsockopt]: %s", strerror(errno));
		throw std::runtime_error("setsockopt");
	}
	if (bind(sockfd, (struct sockaddr *) &sockaddr, sizeof(sockaddr)) < 0) {
		syslog(LOG_ERR, "[bind]: %s", strerror(errno));
		throw std::runtime_error("bind");
	}
	if (listen(sockfd, SOMAXCONN) < 0) {
		close(sockfd);
		syslog(LOG_ERR, "listen");
		throw std::runtime_error("listen");
	}
	if (NULL == inet_ntop(AF_INET, &sockaddr.sin_addr, addr_buf, INET_ADDRSTRLEN)) {
		syslog(LOG_ERR, "[inet_ntop]: %s", strerror(errno));
	}
	syslog(LOG_INFO, "[process: %d]: listening on addr: [%s:%d]", getpid(), addr_buf, port);

}

server_socket::~server_socket() {
	close(sockfd);
}
