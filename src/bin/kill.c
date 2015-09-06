#include <err.h>
#include <errno.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <sysexits.h>

static int signame_to_signo(const char *name);
static void nosig(const char *name);
static void print_signals(FILE *fp);
static void usage(void);

int main(int argc, char *argv[]) {
	int i;
	int pid;
	int signo;
	char *ep;
	int ret;

	if (argc < 2)
		usage();

	signo = SIGTERM;

	--argc, ++argv;
	if (!strcmp(*argv, "-l")) {
		--argc, ++argv;
		if (argc > 1)
			usage();
		if (argc == 1) {
			if (!isdigit(**argv))
				usage();
			signo = strtol(*argv, &ep, 10);
			if (!**argv || *ep)
				errx(2, "illegal signal number: %s", *argv);
			if (signo >= 128)
				signo -= 128;
			if (signo <= 0 || signo >= sys_nsig)
			nosig(*argv);
			printf("%s\n", sys_signame[signo]);
			return EX_OK;
		}
		print_signals(stdout);
		return 0;
	}

	if (!strcmp(*argv, "-s")) {
		--argc, ++argv;
		if (argc < 1) {
			warnx("option requires an argument -- s");
			usage();
		}
		if (strcmp(*argv, "0")) {
			if ((signo = signame_to_signo(*argv)) < 0)
				nosig(*argv);
		} else
			signo = 0;
		--argc, ++argv;
	} else if (**argv == '-' && *(*argv + 1) != '-') {
		++*argv;
		if (isalpha(**argv)) {
			if ((signo = signame_to_signo(*argv)) < 0) {
				nosig(*argv);
			}
		} else if (isdigit(**argv)) {
			signo = strtol(*argv, &ep, 10);
			if (!**argv || *ep)
				errx(2, "illegal signal number: %s", *argv);
			if (signo < 0)
				nosig(*argv);
		} else
			nosig(*argv);
		--argc, ++argv;
	}

	if (argc > 0 && strncmp(*argv, "--", 2) == 0)
		--argc, ++argv;

	if (0 == argc)
		usage();

	for (ret = 0; argc; --argc, ++argv) {
		pid = strtol(*argv, &ep, 10);
		if (!**argv || *ep) {
			errx(2, "illegal process id: %s", *argv);
		}
		ret = kill(pid, signo);
		if (-1 == ret) {
			warn("%s", *argv);
			ret = 1;
		}
	}

	exit(ret);
}

static int signame_to_signo(const char *name) {
	int i;

	if (0 == strncasecmp(name, "SIG", 3))
		name += 3;

	for (i = 1; i < sys_nsig; ++i)
		if (!strcasecmp(sys_signame[i], name))
			return i;
	return -1;
}

static void nosig(const char *name) {
	warnx("unknown signal %s; valid signals:", name);
	print_signals(stderr);
	exit(2);
}

static void print_signals(FILE *fp) {
	int i;

	for (i = 1; i < sys_nsig; ++i) {
		fprintf(fp, "%s", sys_signame[i]);
		if (i == (sys_nsig / 2) || i == (sys_nsig - 1))
			fprintf(fp, "\n");
		else
			fprintf(fp, " ");
	}
}

static void usage(void) {
	fprintf(stderr, "%s\n%s\n%s\n%s\n",
		"usage: kill [-s signal_name] pid ...",
		"	kill -l [exit_status]",
		"	kill -signal_name pid ...",
		"	kill -signal_number pid ...");
	exit(EX_USAGE);
}
