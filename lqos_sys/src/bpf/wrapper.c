#include "wrapper.h"

struct lqos_kern * lqos_kern_open() {
    return lqos_kern__open();
}

int lqos_kern_load(struct lqos_kern * skel) {
    return lqos_kern__load(skel);
}