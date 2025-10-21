/* clang-format off */
/*
        DROP ALL packets at the NIC Level, that are not EtherCAT
*/
#include <linux/types.h>
#include <bpf/bpf_endian.h>
#include <bpf/bpf_helpers.h>
#include <linux/bpf.h>
#include <linux/if_ether.h>
#include <linux/in.h>
#include <stdint.h>

// ether type is expected to be in host byte order
inline int filter_eth_type(struct ethhdr *eth, unsigned short ether_type) {
  return bpf_ntohs(eth->h_proto) == ether_type;
} 

SEC("xdp")
int xtreme_filter(struct xdp_md *ctx) {
  void *data_end = (void *)(long)ctx->data_end;
  void *data = (void *)(long)ctx->data;
  struct ethhdr *eth = data;

  if ((void *)(eth + 1) > data_end)
    return XDP_PASS;

  if (filter_eth_type(eth, ETH_P_ETHERCAT)) {
  }

  return XDP_PASS;
}

char _license[] SEC("license") = "GPL";