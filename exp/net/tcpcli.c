#include <stdio.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <string.h>

#include "service.h"

void send_to_svr(int, const char *);

int main(){
    int sock = socket(AF_INET, SOCK_STREAM, 0);
    if(sock == -1){
        printf("socket init failed\n");
        exit(1);
    }
    struct sockaddr_in svraddr;
    bzero(&svraddr, sizeof(svraddr));
    svraddr.sin_family = AF_INET;
    s_addr = inet_addr(TCP_SRV_ADDR);
    if(s_addr == -1){
        printf("inet_addr failed\n");
        exit(1);
    }
    svraddr.sin_addr.s_addr = s_addr;
    svraddr.sin_port = htons(TCP_SRV_PORT);
    int connection = connect(sock, (struct sockaddr *)&svraddr, sizeof(svraddr));
    if(connection == -1){
        printf("connect failed\n");
        exit(1);
    }
    char sendbuf[REQMAXLEN+1];
    strcpy(sendbuf, "login\0");
    send_to_svr(sock, sendbuf);
    strcpy(sendbuf, "err cmd\0");
    send_to_svr(sock, sendbuf);
    strcpy(sendbuf, "logout\0");
    send_to_svr(sock, sendbuf);
}

void send_to_svr(int sockfd, char * sendbuf){
    char recvbuf[RESMAXLEN+1];
    sendbuf_len = strlen(sendbuf)
    ssize_t length = write(sockfd, sendbuf, sendbuf_len);
    if(length != sendbuf_len){
        printf("write failed, write_len=%d, sendbuflen=%d", length, sendbuf_len);
        exit(1);
    }
    printf("send to server: %s\n", sendbuf);
    ssize_t rec_len = read(sockfd, recvbuf, RESMAXLEN);
    if(rec_len == -1){
        printf("read failed\n");
        exit(1);
    }
    recvbuf[rec_len] = '\0';
    printf("recv from server: [%d] %s\n", rec_len, recvbuf);
}
