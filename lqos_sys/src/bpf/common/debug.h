#pragma once

#define bpf_debug(fmt, ...)                        \
	({                                             \
		char ____fmt[] = " " fmt;             \
		bpf_trace_printk(____fmt, sizeof(____fmt), \
						 ##__VA_ARGS__);           \
	})