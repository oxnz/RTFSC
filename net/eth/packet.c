#include <arpa/inet.h>
#include <netinet/in.h>
#include <netinet/if_ether.h>
#include <sys/socket.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <err.h>

int main(int argc, char *argv[]) {
	int sockfd;
	unsigned char *p;
	int len = 1024;
	unsigned char buf[1024];
	int n;
	int i;

	if ((sockfd = socket(AF_PACKET, SOCK_RAW, htons(ETH_P_IP))) < 0)
		err(1, "socket");

	for (;;) {
		n = recvfrom(sockfd, buf, len, 0, NULL, NULL);
		for (i = 0; i < n; ++i)
			printf("[%02x]", buf[i]);
		putchar('\n');
	}

	exit(0);
}
