#include <stdio.h>
#include <string.h>
#include <inttypes.h>

#include <unistd.h>
#include <fcntl.h>
#include <sys/syscall.h>
#include <linux/aio_abi.h>

inline int io_setup(unsigned nr, aio_context_t *ctxp) {
	return syscall(__NR_io_setup, nr, ctxp);
}

inline int io_destroy(aio_context_t ctx) {
	return syscall(__NR_io_destroy, ctx);
}

inline int io_submit(aio_context_t ctx, long nr, struct iocb **iocbpp) {
	return syscall(__NR_io_submit, ctx, nr, iocbpp);
}

inline int io_getevents(aio_context_t ctx, long min_nr, long max_nr,
		struct io_event *events, struct timespec *timeout) {
	return syscall(__NR_io_getevents, ctx, min_nr, max_nr, events, timeout);
}

int main(int argc, char *argv[]) {
	aio_context_t ctx;
	struct iocb cb;
	struct iocb *cbs[1];
	char data[4096];
	struct io_event events[1];
	int ret;
	int fd;

	fd = open("/tmp/test", O_RDWR | O_CREAT);
	if (fd < 0) {
		perror("open");
		return -1;
	}

	ctx = 0;

	ret = io_setup(128, &ctx);
	if (ret < 0) {
		perror("io_setup");
		return -1;
	}

	/* setup I/O control block */
	memset(&cb, 0, sizeof(cb));
	cb.aio_fildes = fd;
	cb.aio_lio_opcode = IOCB_CMD_PWRITE;

	/* command-specific options */
	int i;
	for (i = 0; i < 4096; ++i)
		data[i] = 'A';
	cb.aio_buf = (uint64_t)data;
	cb.aio_offset = 0;
	cb.aio_nbytes = 4096;

	cbs[0] = &cb;

	ret = io_submit(ctx, 1, cbs);
	if (ret != 1) {
		if (ret < 0) perror("io_submit");
		else fprintf(stderr, "io_submit failed\n");
		return -1;
	}

	/* get reply */
	ret = io_getevents(ctx, 1, 1, events, NULL);
	printf("events: %d\n", ret);
	ret = io_destroy(ctx);
	if (ret < 0) {
		perror("io_destroy");
		return -1;
	}
	return 0;
}
