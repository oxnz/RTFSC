#include <stdio.h>
#include <stdlib.h>
#include <netdb.h>
#include <arpa/inet.h>
#if defined(SOLARIS)
#include <netinet/in.h>
#endif

#if defined(BSD)
#include <sys/socket.h>
#endif

void print_family(struct addrinfo *aip) {
	printf("family ");
	switch (aip->ai_family) {
		case AF_INET:
			printf("inet");
			break;
		case AF_INET6:
			printf("inet6");
			break;
		case AF_UNIX:
			printf("unit");
			break;
		case AF_UNSPEC:
			printf("unspecified");
			break;
		default:
			printf("unknown (%d)", aip->ai_family);
			break;
	}
}

void print_type(struct addrinfo *aip) {
	printf("type: ");
	switch (aip->ai_socktype) {
		case SOCK_STREAM:
			printf("stream");
			break;
		case SOCK_DGRAM:
			printf("datagram");
			break;
		case SOCK_SEQPACKET:
			printf("seqpacket");
			break;
		case SOCK_RAW:
			printf("raw");
			break;
		default:
			printf("unknown (%d)", aip->ai_socktype);
			break;
	}
}

void print_proto(struct addrinfo *aip) {
	printf("protocol: ");
	switch (aip->ai_protocol) {
		case 0:
			printf("default");
			break;
		case IPPROTO_TCP:
			printf("tcp");
			break;
		case IPPROTO_UDP:
			printf("udp");
			break;
		case IPPROTO_RAW:
			printf("raw");
			break;
		default:
			printf("unknown (%d)", aip->ai_protocol);
			break;
	}
}

void print_flags(struct addrinfo *aip) {
	printf("flags: ");
	if (aip->ai_flags == 0) {
		printf("none");
	} else if (aip->ai_flags & AI_ALL) {
		printf("all");
	} else {
		if (aip->ai_flags & AI_PASSIVE)
			printf("passive");
		if (aip->ai_flags & AI_CANONNAME)
			printf("canon");
		if (aip->ai_flags & AI_NUMERICSERV)
			printf("numhost");
		if (aip->ai_flags & AI_NUMERICHOST)
			printf("numserv");
		if (aip->ai_flags & AI_V4MAPPED)
			printf("v4mapped");
	}
}

void print_addrinfo(struct addrinfo *aip) {
	print_flags(aip);
	putchar('\n');
	print_family(aip);
	putchar('\n');
	print_type(aip);
	putchar('\n');
	print_proto(aip);
	putchar('\n');
	printf("host: %s", aip->ai_canonname ? aip->ai_canonname : "-");
	putchar('\n');
	if (aip->ai_family == AF_INET) {
		struct sockaddr_in *sinp = (struct sockaddr_in *)aip->ai_addr;
		char abuf[INET_ADDRSTRLEN];
		const char *addr = inet_ntop(AF_INET, &sinp->sin_addr, abuf,
				INET_ADDRSTRLEN);
		printf("address: %s\n", addr ? addr : "unknown");
		printf("port: %d\n", ntohs(sinp->sin_port));
	}
	putchar('\n');
}

int main(int argc, char *argv[]) {
	struct addrinfo *ailist, *aip;
	struct addrinfo hint;
	struct sockaddr_in *sinp;
	const char *addr;
	int err;

	if (argc != 3) {
		fprintf(stderr, "usage: %s nodename service\n", argv[0]);
		exit(1);
	}

	hint.ai_flags = AI_CANONNAME;
	hint.ai_family = 0;
	hint.ai_socktype = 0;
	hint.ai_protocol = 0;
	hint.ai_canonname = NULL;
	hint.ai_addr = NULL;
	hint.ai_next = NULL;
	if (err = getaddrinfo(argv[1], argv[2], &hint, &ailist)) {
		fprintf(stderr, "getaddrinfo: %s", gai_strerror(err));
		exit(1);
	}
	for (aip = ailist; aip; aip = aip->ai_next) {
		print_addrinfo(aip);
	}

	exit(0);
}
