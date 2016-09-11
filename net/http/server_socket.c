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

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include "server_socket.h"

int server_socket_init(struct server_socket *skt) {
	struct sockaddr_in sockaddr;
	char addr_buf[INET_ADDRSTRLEN];
	int optval = 1;

	if ((skt->sockfd = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP)) < 0) {
		perror("socket");
		return -1;
	}
	{ // setup addr
		memset(&sockaddr, 0, sizeof(sockaddr));
		sockaddr.sin_family = AF_INET;
		sockaddr.sin_addr.s_addr = htonl(skt->addr);
		sockaddr.sin_port = htons(skt->port);
	}

	if (setsockopt(skt->sockfd, SOL_SOCKET, SO_REUSEPORT,
				&optval, sizeof(optval)) < 0) {
		perror("setsockopt");
		return -1;
	}
	if (bind(skt->sockfd, (struct sockaddr *) &sockaddr, sizeof(sockaddr)) < 0) {
		perror("bind");
		return -1;
	}
	if (listen(skt->sockfd, SOMAXCONN) < 0) {
		close(skt->sockfd);
		perror("listen");
		return -1;
	}
	if (inet_ntop(AF_INET, &sockaddr.sin_addr, addr_buf, INET_ADDRSTRLEN)) {
		printf("[%d] listening on addr: [%s:%d]\n", getpid(), addr_buf, skt->port);
	} else
		printf("[%d] listening on addr: [misc:%d]\n", getpid(), skt->port);

	return 0;
}

int server_socket_destroy(struct server_socket *skt) {
	return close(skt->sockfd);
}
