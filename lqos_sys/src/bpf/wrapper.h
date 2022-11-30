#include "lqos_kern_skel.h"

extern struct lqos_kern * lqos_kern_open();
extern int lqos_kern_load(struct lqos_kern * skel);
extern int tc_attach_egress(int ifindex, bool verbose, struct lqos_kern *obj);
extern int tc_detach_egress(int ifindex, bool verbose, bool flush_hook, const char * ifname);