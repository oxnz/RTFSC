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
	int sockfd, n, len;
	const char *prompt = "msg: ";
	char req[REQMAXLEN], rsp[RSPMAXLEN];

	if ((sockfd = socket(PF_INET, SOCK_DGRAM, IPPROTO_UDP)) < 0)
		err(1, "socket");
	memset(&srv_addr, 0, sizeof(srv_addr));
	srv_addr.sin_family = AF_INET;
	srv_addr.sin_addr.s_addr = htonl(INADDR_ANY);
	srv_addr.sin_port = htons(UDP_SERVICE_PORT);
	if (bind(sockfd, (SA) &srv_addr, sizeof(srv_addr)) < 0)
		err(1, "bind");

	for (;;) {
		len = sizeof(cli_addr);
		if ((n = recvfrom(sockfd, req, REQMAXLEN, 0,
					(SA) &cli_addr, &len)) < 0)
			err(1, "recvfrom");
		printf("[msg len: %d]: [%s]\n", n, req);
		write(1, prompt, strlen(prompt));
		if ((n = read(1, rsp, RSPMAXLEN)) < 0)
			err(1, "read");
		if (sendto(sockfd, rsp, n, 0,
					(SA) &cli_addr, sizeof(cli_addr)) != n)
			err(1, "sendto");
	}

	exit(0);
}
