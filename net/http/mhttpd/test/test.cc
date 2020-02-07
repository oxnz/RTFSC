#include <stdio.h>
#include <unistd.h>
 #include <sys/types.h>
       #include <sys/stat.h>
       #include <fcntl.h>

int main() {
		for (int i = 0; i < 100; ++i) {
				int fd = open("index.html", O_RDONLY);
				printf("fd = %d\n", fd);
				//close(fd);
		}
		return 0;
}
