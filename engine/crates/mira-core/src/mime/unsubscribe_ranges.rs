//! Plages d'adresses IP jamais jointes par le désabonnement (loopback, privées, réservées, multicast).

use std::net::{Ipv4Addr, Ipv6Addr};

/// Vrai pour une adresse IPv4 de boucle, privée, locale, partagée, réservée ou de diffusion.
pub(super) fn is_private_v4(ip: Ipv4Addr) -> bool {
    let [a, b, c, _] = ip.octets();
    ip.is_loopback()
        || ip.is_private()
        || ip.is_link_local()
        || ip.is_unspecified()
        || ip.is_broadcast()
        || a >= 240
        || (a == 100 && (64..128).contains(&b))
        || (a == 198 && (b == 18 || b == 19))
        || (a == 192 && b == 0 && c == 0)
}

/// Vrai pour une adresse IPv6 de boucle, locale, unique-locale, multicast ou IPv4 embarquée privée.
pub(super) fn is_private_v6(ip: Ipv6Addr) -> bool {
    let s = ip.segments();
    ip.is_loopback()
        || ip.is_unspecified()
        || s[0] & 0xfe00 == 0xfc00
        || s[0] & 0xffc0 == 0xfe80
        || s[0] & 0xffc0 == 0xfec0
        || s[0] & 0xff00 == 0xff00
        || s[..6].iter().all(|&x| x == 0)
        || ip.to_ipv4_mapped().is_some_and(is_private_v4)
        || is_private_nat64(s)
}

/// Vrai pour une adresse NAT64 64:ff9b::/96 dont l'IPv4 embarquée est privée ou réservée.
fn is_private_nat64(segments: [u16; 8]) -> bool {
    let prefix = [0x64, 0xff9b, 0, 0, 0, 0];
    segments[..6] == prefix && {
        let [a, b] = segments[6].to_be_bytes();
        let [c, d] = segments[7].to_be_bytes();
        is_private_v4(Ipv4Addr::new(a, b, c, d))
    }
}
