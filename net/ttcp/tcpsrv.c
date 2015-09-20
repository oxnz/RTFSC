#include <sys/types.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <sys/socket.h>

#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "service.h"

int main(int argc, char *argv[]) {
	struct sockaddr_in srv_addr, cli_addr;
	int listenfd, sockfd, n, len;
	const char *prompt = "msg: ";
	char req[REQMAXLEN], rsp[RSPMAXLEN];

	if ((listenfd = socket(PF_INET, SOCK_STREAM, IPPROTO_TCP)) < 0)
		err(1, "socket");
	memset(&srv_addr, 0, sizeof(srv_addr));
	srv_addr.sin_family = AF_INET;
	srv_addr.sin_addr.s_addr = htonl(INADDR_ANY);
	srv_addr.sin_port = htons(TCP_SERVICE_PORT);
	if (bind(listenfd, (SA) &srv_addr, sizeof(srv_addr)) < 0)
		err(1, "bind");
	if (listen(listenfd, SOMAXCONN) < 0)
		err(1, "listen");

	for (;;) {
		len = sizeof(cli_addr);
		if ((sockfd = accept(listenfd, (SA) &cli_addr, &len)) < 0)
			err(1, "accept");
		if ((n = read(sockfd, req, REQMAXLEN)) < 0)
			err(1, "read");
		printf("[%s:%u]: %s", inet_ntoa(*((struct in_addr *)&cli_addr.sin_addr.s_addr)), ntohs(cli_addr.sin_port), req);
		write(1, prompt, strlen(prompt));
		if ((n = read(1, rsp, RSPMAXLEN)) < 0)
			err(1, "read");
		if (write(sockfd, rsp, n) != n)
			err(1, "write");
		close(sockfd);
	}

	exit(0);
}
