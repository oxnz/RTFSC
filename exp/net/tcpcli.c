#include <stdio.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <string.h>

#include "service.h"

void client(int);
void send_to_svr(int, char *);

int main(){
    printf("hello\n");
    client(1);
}

void client(int clit_num){
    printf("clit num = %d\n", clit_num);
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    struct sockaddr_in svraddr;
    bzero(&svraddr, sizeof(svraddr));
    svraddr.sin_family = AF_INET;
    svraddr.sin_addr.s_addr = inet_addr(TCP_SRV_ADDR);
    svraddr.sin_port = htons(TCP_SRV_PORT);
    int connection = connect(sock, (struct sockaddr *)&svraddr, sizeof(svraddr));
    char sendbuf[REQMAXLEN];
    strcpy(sendbuf, "login\0");
    send_to_svr(sock, sendbuf);
    strcpy(sendbuf, "err cmd\0");
    send_to_svr(sock, sendbuf);
    strcpy(sendbuf, "logout\0");
    send_to_svr(sock, sendbuf);
}

void send_to_svr(int sockfd, char * sendbuf){
    char recvbuf[RESMAXLEN];
    int length = write(sockfd, sendbuf, strlen(sendbuf));
    printf("send to server: %s\n", sendbuf);
    ssize_t rec_len = read(sockfd, recvbuf, RESMAXLEN);
    recvbuf[rec_len] = '\0';
    printf("recv from server: [%d] %s\n", rec_len, recvbuf);
}
