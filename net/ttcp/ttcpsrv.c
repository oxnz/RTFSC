#include <sys/types.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <err.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

#include "service.h"

int main(int argc, char *argv[]) {
	struct sockaddr_in srv_addr, cli_addr;
	int sockfd, clifd, n, len;
	const char *prompt = "msg: ";
	char req[REQMAXLEN], rsp[RSPMAXLEN];
	char *pbuf = req;

	if ((sockfd = socket(PF_INET, SOCK_DGRAM, IPPROTO_UDP)) < 0)
		err(1, "socket");
	memset(&srv_addr, 0, sizeof(srv_addr));
	srv_addr.sin_family = AF_INET;
	srv_addr.sin_addr.s_addr = htonl(INADDR_ANY);
	srv_addr.sin_port = htons(TTCP_SERVICE_PORT);
	if (bind(sockfd, (SA) &srv_addr, sizeof(srv_addr)) < 0)
		err(1, "bind");
	if (listen(sockfd, SOMAXCONN) < 0)
		err(1, "listen");

	for (;;) {
		len = sizeof(cli_addr);
		if ((clifd = accept(sockfd, (SA) &cli_addr, &len)) < 0)
			err(1, "accept");
		while ((n = read(clifd, pbuf, REQMAXLEN-(pbuf - req))) > 0) {
			pbuf += n;
		}
		if (n < 0)
			err(1, "read");
		printf("[msg len: %d]: [%s]\n", pbuf-req, req);
		printf("%s", prompt);
		if ((n = read(1, rsp, RSPMAXLEN)) < 0)
			err(1, "read");
		if (send(sockfd, rsp, n, MSG_EOF) != n)
			err(1, "sendto");
		close(clifd);
	}

	exit(0);
}
